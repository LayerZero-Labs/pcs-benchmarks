#include <stdint.h>
#include <stdlib.h>

#include "fips202.h"
#include "randombytes.h"

/*
 * Deterministic benchmark-only randomness. Production Greyhound continues to
 * use the upstream OS-backed randombytes.c; the benchmark build substitutes
 * this file so a recorded PCS_BENCH_SEED identifies the complete workload and
 * prover-randomness stream.
 */
void randombytes(uint8_t *out, size_t outlen) {
    static uint64_t counter = 0;
    static uint64_t seed = 0;
    static int initialized = 0;

    if (!initialized) {
        const char *seed_text = getenv("PCS_BENCH_SEED");
        if (seed_text && seed_text[0] != '\0') {
            char *end = NULL;
            seed = strtoull(seed_text, &end, 10);
            if (end == seed_text || *end != '\0') {
                abort();
            }
        } else {
            seed = 0x50435342454e4348ull;
        }
        initialized = 1;
    }

    uint8_t input[16];
    for (size_t i = 0; i < 8; i++) {
        input[i] = (uint8_t)(seed >> (8 * i));
        input[8 + i] = (uint8_t)(counter >> (8 * i));
    }
    counter++;
    shake256(out, outlen, input, sizeof(input));
}
