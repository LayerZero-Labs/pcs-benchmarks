/* Greyhound Pack worker: commit / open / verify one dense instance.
 *
 * Pins LayerZero greyhound-reference. The runner and this binary force
 * LATTICE_DOGS_THREADS=1 and LABRADOR_SIS_SECURITY=l2-quantum128-adps16 so
 * lattice-eval stays single-threaded and uses the 128-bit ADPS16 Euclidean
 * SIS policy. Proof bytes are the contextual wire encoding (public u1 and
 * the fold schedule are verifier context, not charged to the proof).
 */

#define _POSIX_C_SOURCE 200809L

#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <sys/resource.h>

#include "malloc.h"
#include "randombytes.h"
#include "data.h"
#include "labrador.h"
#include "chihuahua.h"
#include "pack.h"
#include "greyhound.h"

static uint64_t monotonic_ns(void) {
    struct timespec now;
    if (clock_gettime(CLOCK_MONOTONIC, &now) != 0) {
        return 0;
    }
    return (uint64_t)now.tv_sec * 1000000000ull + (uint64_t)now.tv_nsec;
}

static uint64_t peak_rss_bytes(void) {
#ifdef __linux__
    struct rusage usage;
    if (getrusage(RUSAGE_SELF, &usage) != 0) {
        return 0;
    }
    /* Linux reports ru_maxrss in kilobytes. */
    return (uint64_t)usage.ru_maxrss * 1024ull;
#else
    return 0;
#endif
}

static int parse_log2_n(int argc, char **argv, uint32_t *log2_n) {
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--log2-n") == 0 && i + 1 < argc) {
            *log2_n = (uint32_t)strtoul(argv[i + 1], NULL, 10);
            return 0;
        }
    }
    return 1;
}

static int negative_check_enabled(void) {
    const char *value = getenv("PCS_BENCH_NEGATIVE_CHECK");
    return value && strcmp(value, "1") == 0;
}

static const char *commit_error(int ret) {
    if (ret == 1) {
        return "Cannot make inner commitments secure";
    }
    if (ret == 2) {
        return "Cannot make outer commitments secure";
    }
    return "polcom_commit failed";
}

static void emit(
    const char *status,
    const char *detail,
    uint32_t log2_n,
    uint64_t commit_ns,
    uint64_t open_ns,
    uint64_t verify_ns,
    uint64_t proof_bytes,
    uint64_t commitment_bytes,
    int have_timings
) {
    printf("{\"status\":\"%s\"", status);
    if (detail && detail[0] != '\0') {
        printf(",\"status_detail\":\"%s\"", detail);
    }
    printf(",\"log2_n\":%" PRIu32, log2_n);
    if (have_timings) {
        printf(
            ",\"timings_ns\":{\"commit\":%" PRIu64 ",\"open\":%" PRIu64 ",\"verify\":%" PRIu64 "}",
            commit_ns,
            open_ns,
            verify_ns
        );
        printf(",\"proof_bytes\":%" PRIu64, proof_bytes);
        printf(",\"commitment_bytes\":%" PRIu64, commitment_bytes);
        printf(",\"evaluation_bytes\":8");
        printf(",\"public_context_bytes\":0");
        printf(",\"state_bytes\":%" PRIu64, (uint64_t)comkey_len * (uint64_t)sizeof(polx));
    }
    printf(",\"peak_rss_bytes\":%" PRIu64 "}\n", peak_rss_bytes());
}

int main(int argc, char **argv) {
    /* Headline lattice-eval is single-threaded and uses the 128-bit ADPS16
     * Euclidean SIS policy, even if the parent shell exported something else. */
    if (setenv("LATTICE_DOGS_THREADS", "1", 1) != 0
        || setenv("LABRADOR_SIS_SECURITY", "l2-quantum128-adps16", 1) != 0) {
        emit("error", "failed to set Greyhound lattice-eval environment", 0, 0, 0, 0, 0, 0, 0);
        return 1;
    }

    uint32_t log2_n = 0;
    if (parse_log2_n(argc, argv, &log2_n) != 0 || log2_n < 6 || log2_n >= 63) {
        emit("error", "missing or invalid --log2-n (need >= 6)", log2_n, 0, 0, 0, 0, 0, 0);
        return 1;
    }

    const size_t len = (size_t)1 << (log2_n - 6);
    polz *s = _aligned_alloc(64, len * sizeof(polz));
    if (s == NULL) {
        emit("oom", "polynomial allocation failed", log2_n, 0, 0, 0, 0, 0, 0);
        return 1;
    }

    /* The statement is fixed across legitimate prover retries. */
    uint8_t seed[16] __attribute__((aligned(16)));
    const char *seed_text = getenv("PCS_BENCH_SEED");
    if (seed_text && seed_text[0] != '\0') {
        char *end = NULL;
        const uint64_t workload_seed = strtoull(seed_text, &end, 10);
        if (end == seed_text || *end != '\0') {
            emit("error", "invalid PCS_BENCH_SEED", log2_n, 0, 0, 0, 0, 0, 0);
            free(s);
            return 1;
        }
        const uint64_t second = workload_seed ^ 0x9e3779b97f4a7c15ull;
        for (size_t i = 0; i < 8; i++) {
            seed[i] = (uint8_t)(workload_seed >> (8 * i));
            seed[8 + i] = (uint8_t)(second >> (8 * i));
        }
    } else {
        randombytes(seed, 16);
    }
    polzvec_almostuniform(s, len, seed, 0);
    polzvec_center(s, len);

    const int64_t x = 43;
    int64_t y = 0;

    /* The pinned path is deterministic after the statement is fixed. Treat a
     * prover failure as a failed observation instead of repeating identical work. */
    enum { kMaxAttempts = 1 };
    uint64_t total_commit_ns = 0;
    uint64_t total_open_ns = 0;
    uint64_t total_verify_ns = 0;

    for (int attempt = 0; attempt < kMaxAttempts; attempt++) {
        polcomctx ctx = {0};
        polcomprf pi = {0};
        composite proof = {0};

        uint64_t t0 = monotonic_ns();
        int ret = polcom_commit(&ctx, s, len);
        uint64_t commit_ns = monotonic_ns() - t0;
        total_commit_ns += commit_ns;
        if (ret) {
            char detail[160];
            snprintf(
                detail,
                sizeof(detail),
                "%s; attempts=%d; reusable-key setup is embedded in commit",
                commit_error(ret),
                attempt + 1
            );
            emit(
                "error",
                detail,
                log2_n,
                total_commit_ns,
                total_open_ns,
                total_verify_ns,
                0,
                0,
                1
            );
            free(s);
            free_comkey();
            return 1;
        }

        t0 = monotonic_ns();
        y = polzvec_eval(s, len, x);
        total_open_ns += monotonic_ns() - t0;

        t0 = monotonic_ns();
        ret = composite_prove_polcom(&proof, &pi, &ctx, (uint32_t)x, (uint32_t)y);
        uint64_t open_ns = monotonic_ns() - t0;
        total_open_ns += open_ns;
        if (ret) {
            free_polcomctx(&ctx);
            free_polcomprf(&pi);
            free_composite(&proof);
            continue;
        }

        t0 = monotonic_ns();
        ret = composite_verify_polcom(&proof, &pi);
        uint64_t verify_ns = monotonic_ns() - t0;
        total_verify_ns += verify_ns;
        const uint64_t proof_bytes =
            (uint64_t)greyhound_pack_contextual_serialized_size(&pi, &proof);
        /* Public commitment u1 (verifier context). u2 lives in the proof. */
        const uint64_t commitment_bytes =
            (uint64_t)ctx.cpp->kappa1 * (uint64_t)N * (uint64_t)QBYTES;
        if (ret) {
            char detail[160];
            snprintf(
                detail,
                sizeof(detail),
                "composite_verify_polcom failed; attempts=%d; statement retained",
                attempt + 1
            );
            emit(
                "error",
                detail,
                log2_n,
                total_commit_ns,
                total_open_ns,
                total_verify_ns,
                proof_bytes,
                commitment_bytes,
                1
            );
            free_polcomctx(&ctx);
            free_polcomprf(&pi);
            free_composite(&proof);
            free_comkey();
            free(s);
            return 1;
        }

        if (negative_check_enabled()) {
            const int64_t valid_y = pi.y;
            pi.y = valid_y + 1;
            ret = composite_verify_polcom(&proof, &pi);
            pi.y = valid_y;
            if (ret == 0) {
                emit(
                    "error",
                    "Greyhound verifier accepted an altered opening claim",
                    log2_n,
                    total_commit_ns,
                    total_open_ns,
                    total_verify_ns,
                    proof_bytes,
                    commitment_bytes,
                    1
                );
                free(s);
                free_polcomctx(&ctx);
                free_polcomprf(&pi);
                free_composite(&proof);
                free_comkey();
                return 1;
            }
        }

        char detail[160];
        snprintf(
            detail,
            sizeof(detail),
            "attempts=%d; statement retained; distribution=bounded-native; point=fixed-43; evaluation=separate; reusable-key setup is embedded in commit",
            attempt + 1
        );
        emit(
            "ok",
            detail,
            log2_n,
            total_commit_ns,
            total_open_ns,
            total_verify_ns,
            proof_bytes,
            commitment_bytes,
            1
        );
        free(s);
        free_polcomctx(&ctx);
        free_polcomprf(&pi);
        free_composite(&proof);
        free_comkey();
        return 0;
    }

    char detail[160];
    snprintf(
        detail,
        sizeof(detail),
        "composite_prove_polcom exhausted %d attempts for one retained statement",
        kMaxAttempts
    );
    emit(
        "error",
        detail,
        log2_n,
        total_commit_ns,
        total_open_ns,
        total_verify_ns,
        0,
        0,
        1
    );
    free(s);
    free_comkey();
    return 1;
}
