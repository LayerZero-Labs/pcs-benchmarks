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
    struct rusage usage;
    if (getrusage(RUSAGE_SELF, &usage) != 0) {
        return 0;
    }
    /* Linux reports ru_maxrss in kilobytes. */
    return (uint64_t)usage.ru_maxrss * 1024ull;
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
            ",\"timings_ns\":{\"setup\":0,\"commit\":%" PRIu64 ",\"open\":%" PRIu64 ",\"verify\":%" PRIu64 "}",
            commit_ns,
            open_ns,
            verify_ns
        );
        printf(",\"proof_bytes\":%" PRIu64, proof_bytes);
        printf(",\"commitment_bytes\":%" PRIu64, commitment_bytes);
        printf(",\"state_bytes\":0");
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
    /* SIS search and fold grinding are randomized; a prove/verify pair can
     * fail even at supported sizes. Retry with a fresh polynomial and do not
     * include failed attempts in the timings. */
    enum { kMaxAttempts = 8 };
    const char *last_error = "greyhound attempts exhausted";

    for (int attempt = 0; attempt < kMaxAttempts; attempt++) {
        polz *s = _aligned_alloc(64, len * sizeof(polz));
        if (s == NULL) {
            emit("oom", "polynomial allocation failed", log2_n, 0, 0, 0, 0, 0, 0);
            return 1;
        }

        uint8_t seed[16] __attribute__((aligned(16)));
        randombytes(seed, 16);
        polzvec_almostuniform(s, len, seed, 0);
        polzvec_center(s, len);

        const int64_t x = 43;
        const int64_t y = polzvec_eval(s, len, x);

        polcomctx ctx = {0};
        polcomprf pi = {0};
        composite proof = {0};

        uint64_t t0 = monotonic_ns();
        int ret = polcom_commit(&ctx, s, len);
        uint64_t commit_ns = monotonic_ns() - t0;
        if (ret) {
            last_error = commit_error(ret);
            free(s);
            free_comkey();
            continue;
        }

        t0 = monotonic_ns();
        ret = composite_prove_polcom(&proof, &pi, &ctx, (uint32_t)x, (uint32_t)y);
        uint64_t open_ns = monotonic_ns() - t0;
        if (ret) {
            last_error = "composite_prove_polcom failed";
            free(s);
            free_polcomctx(&ctx);
            free_polcomprf(&pi);
            free_composite(&proof);
            free_comkey();
            continue;
        }

        t0 = monotonic_ns();
        ret = composite_verify_polcom(&proof, &pi);
        uint64_t verify_ns = monotonic_ns() - t0;
        const uint64_t proof_bytes =
            (uint64_t)greyhound_pack_contextual_serialized_size(&pi, &proof);
        /* Public commitment u1 (verifier context). u2 lives in the proof. */
        const uint64_t commitment_bytes =
            (uint64_t)ctx.cpp->kappa1 * (uint64_t)N * (uint64_t)QBYTES;
        if (ret) {
            last_error = "composite_verify_polcom failed";
            free(s);
            free_polcomctx(&ctx);
            free_polcomprf(&pi);
            free_composite(&proof);
            free_comkey();
            continue;
        }

        emit("ok", "", log2_n, commit_ns, open_ns, verify_ns, proof_bytes, commitment_bytes, 1);
        free(s);
        free_polcomctx(&ctx);
        free_polcomprf(&pi);
        free_composite(&proof);
        free_comkey();
        return 0;
    }

    emit("error", last_error, log2_n, 0, 0, 0, 0, 0, 0);
    return 1;
}
