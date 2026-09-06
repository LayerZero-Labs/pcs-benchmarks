#![allow(missing_docs)]

use akita_algebra::poly::multilinear_eval;
use akita_config::proof_optimized::fp128;
use akita_config::CommitmentConfig;
use akita_pcs::AkitaCommitmentScheme;
use akita_prover::{ComputeBackendSetup, CpuBackend, DensePoly, SelectedProverOpeningData};
use akita_transcript::AkitaTranscript;
use akita_types::{
    AkitaCommitmentHint, BasisMode, CommittedGroup, CommittedGroupBatchProfile,
    GroupBatchStatement, OpeningClaims, OpeningScheduleSelection, PolynomialGroupClaims,
};
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use jolt_field::{CanonicalEncoding, Ring, Zero};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::time::Duration;

type F = fp128::Field;

const INPUT_SEED: u64 = 0xDEAD_BEEF;
const POINT_SEED: u64 = 0xCAFE_BABE;

fn dense_evaluations<Cfg: CommitmentConfig<Field = F>>(num_vars: usize) -> Vec<F> {
    let mut rng = StdRng::seed_from_u64(INPUT_SEED);
    let decomposition = Cfg::decomposition();
    if decomposition.log_commit_bound >= 128 {
        (0..(1usize << num_vars))
            .map(|_| F::from_u128_reduced(rng.gen::<u128>()))
            .collect()
    } else {
        let half_bound = 1i64 << (decomposition.log_commit_bound.min(62) - 1);
        (0..(1usize << num_vars))
            .map(|_| F::from_i64(rng.gen_range(-half_bound..half_bound)))
            .collect()
    }
}

fn opening_point(num_vars: usize) -> Vec<F> {
    let mut rng = StdRng::seed_from_u64(POINT_SEED);
    (0..num_vars)
        .map(|_| F::from_u128_reduced(rng.gen::<u128>()))
        .collect()
}

fn prover_claims<'a, Cfg, P>(
    point: &'a [F],
    polynomials: &'a [&'a P],
    commitment: &'a CommittedGroup<Cfg::Field>,
    hint: AkitaCommitmentHint<Cfg::Field>,
) -> SelectedProverOpeningData<'a, F, akita_prover::PreparedProverGroup<'a, P>, Cfg::Field>
where
    Cfg: CommitmentConfig<ExtField = F>,
    P: akita_prover::RootPolyMeta<Cfg::Field>,
{
    let group = PolynomialGroupClaims::new(
        point.to_vec(),
        vec![F::zero(); polynomials.len()],
        commitment.clone(),
    )
    .expect("benchmark claims are valid");
    let claims = OpeningClaims::from_groups(vec![group]).expect("benchmark claim group is valid");
    SelectedProverOpeningData::from_committed_claims::<Cfg>(claims, vec![hint], vec![polynomials])
        .expect("benchmark prover data is valid")
}

fn verifier_claims<'a>(
    selection: OpeningScheduleSelection,
    point: &[F],
    openings: &[F],
    commitment: &'a CommittedGroup<F>,
) -> GroupBatchStatement<'a, F, F> {
    let group = PolynomialGroupClaims::new(point.to_vec(), openings.to_vec(), commitment)
        .expect("benchmark verifier claims are valid");
    let claims = OpeningClaims::from_groups(vec![group]).expect("benchmark claims are valid");
    GroupBatchStatement::new(selection, claims).expect("benchmark statement is valid")
}

#[allow(clippy::too_many_lines)] // Keeping one fixture scope prevents timed-state drift.
fn bench_dense<Cfg>(criterion: &mut Criterion, num_vars: usize)
where
    Cfg: CommitmentConfig<Field = F, ExtField = F>,
{
    let evaluations = dense_evaluations::<Cfg>(num_vars);
    let polynomial =
        DensePoly::<F>::from_field_evals(num_vars, &evaluations).expect("valid dense polynomial");
    let point = opening_point(num_vars);
    let opening = multilinear_eval(&evaluations, &point).expect("valid multilinear evaluation");

    let mut group = criterion.benchmark_group(format!("akita/fp128/dense/nv{num_vars}"));
    group.throughput(Throughput::Elements(1u64 << num_vars));
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    group.bench_function("setup", |bencher| {
        bencher.iter(|| {
            black_box(
                AkitaCommitmentScheme::<Cfg>::setup_prover(black_box(num_vars), black_box(1))
                    .expect("setup succeeds"),
            )
        });
    });

    let setup = AkitaCommitmentScheme::<Cfg>::setup_prover(num_vars, 1).expect("setup succeeds");
    let prepared = CpuBackend::DEFAULT
        .prepare_setup(&setup)
        .expect("setup preparation succeeds");
    let stack = akita_prover::UniformProverStack::uniform(
        &CpuBackend::DEFAULT,
        &prepared,
        setup.expanded.as_ref(),
    )
    .expect("prover stack construction succeeds");

    group.bench_function("commit", |bencher| {
        bencher.iter(|| {
            black_box(
                AkitaCommitmentScheme::<Cfg>::commit::<_, _>(
                    &setup,
                    black_box(std::slice::from_ref(&polynomial)),
                    &stack,
                    akita_prover::GroupContext::scheduler_without_precommitted_groups(),
                )
                .expect("commit succeeds"),
            )
        });
    });

    let output = AkitaCommitmentScheme::<Cfg>::commit::<_, _>(
        &setup,
        std::slice::from_ref(&polynomial),
        &stack,
        akita_prover::GroupContext::scheduler_without_precommitted_groups(),
    )
    .expect("commit succeeds");
    let polynomial_refs = [&polynomial];
    let selection = Cfg::resolve_catalog_row_for_profiles(&CommittedGroupBatchProfile {
        final_group: *output.committed_group.profile(),
        precommitteds: Vec::new(),
    })
    .expect("generated schedule contains benchmark case")
    .selection();
    let verifier_setup =
        AkitaCommitmentScheme::<Cfg>::setup_verifier(&setup).expect("verifier setup succeeds");

    group.bench_function("prove", |bencher| {
        bencher.iter_batched(
            || output.hint.clone(),
            |hint| {
                let mut transcript = AkitaTranscript::<F>::new(b"pcs-benchmark/v1");
                black_box(
                    AkitaCommitmentScheme::<Cfg>::batched_prove::<_, _, _>(
                        &setup,
                        prover_claims::<Cfg, _>(
                            &point,
                            &polynomial_refs,
                            &output.committed_group,
                            hint,
                        ),
                        &stack,
                        &mut transcript,
                        BasisMode::Lagrange,
                    )
                    .expect("proving succeeds"),
                )
            },
            BatchSize::LargeInput,
        );
    });

    let mut prover_transcript = AkitaTranscript::<F>::new(b"pcs-benchmark/v1");
    let proof = AkitaCommitmentScheme::<Cfg>::batched_prove::<_, _, _>(
        &setup,
        prover_claims::<Cfg, _>(
            &point,
            &polynomial_refs,
            &output.committed_group,
            output.hint.clone(),
        ),
        &stack,
        &mut prover_transcript,
        BasisMode::Lagrange,
    )
    .expect("proving succeeds");

    group.bench_function("verify", |bencher| {
        bencher.iter(|| {
            let mut transcript = AkitaTranscript::<F>::new(b"pcs-benchmark/v1");
            AkitaCommitmentScheme::<Cfg>::batched_verify(
                black_box(&proof),
                black_box(&verifier_setup),
                &mut transcript,
                black_box(verifier_claims(
                    selection,
                    &point,
                    &[opening],
                    &output.committed_group,
                )),
                BasisMode::Lagrange,
            )
            .expect("verification succeeds");
        });
    });

    group.finish();
}

fn dense_nv14(criterion: &mut Criterion) {
    bench_dense::<fp128::Dense>(criterion, 14);
}

fn dense_nv16(criterion: &mut Criterion) {
    bench_dense::<fp128::Dense>(criterion, 16);
}

criterion_group!(akita, dense_nv14, dense_nv16);
criterion_main!(akita);
