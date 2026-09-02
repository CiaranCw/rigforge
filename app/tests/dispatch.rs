mod common;

use common::{certify, unpublished_graph, unpublished_graph_with_time};
use rigforge_app::{AppError, FakeWorker, SqliteCatalog, WorkerDispatchRequest};
use rigforge_domain::{
    from_json_validated, SamplingInterpretation, TimeDomainProvenance, TimePoint, Validated,
    MotionAssetVersion,
};

fn seed_graph(catalog: &mut SqliteCatalog, g: &common::Graph) -> Validated<rigforge_domain::JobSpec> {
    catalog
        .put_validated_pair(
            &certify(g.character.clone()),
            &certify(g.character_version.clone()),
        )
        .unwrap();
    catalog
        .put_validated(&certify(g.source_skeleton.clone()))
        .unwrap();
    catalog
        .put_validated_pair(
            &certify(g.motion.clone()),
            &certify(g.motion_version.clone()),
        )
        .unwrap();
    catalog
        .put_validated(&certify(g.mapping_version.clone()))
        .unwrap();
    catalog
        .put_validated(&certify(g.policy_version.clone()))
        .unwrap();
    catalog
        .put_validated(&certify(g.compatibility.clone()))
        .unwrap();
    certify(g.job.clone())
}

fn assemble_for_graph(
    catalog: &mut SqliteCatalog,
    g: &common::Graph,
) -> Result<WorkerDispatchRequest, AppError> {
    let spec = seed_graph(catalog, g);
    let run = catalog.enqueue_job(spec).unwrap();
    catalog.assemble_worker_dispatch_request(&run)
}

fn fractional_time(start: TimePoint, end: TimePoint) -> TimeDomainProvenance {
    TimeDomainProvenance::new(
        "clip:walk-carry",
        start,
        end,
        SamplingInterpretation::BakedEverySourceFrame,
        "unmapped target joints remain at target rest",
    )
    .unwrap()
}

#[test]
fn fractional_start_frame_rejected_before_dispatch() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = unpublished_graph_with_time(fractional_time(
        TimePoint::frames_rational(3, 2, 30, 1).unwrap(),
        TimePoint::frames(4, 30, 1).unwrap(),
    ));
    let err = assemble_for_graph(&mut catalog, &g).unwrap_err();
    match err {
        AppError::Orchestration(msg) => {
            assert!(msg.contains("unsupported execution-time provenance"), "{msg}");
            assert!(msg.contains("value_den"), "{msg}");
        }
        other => panic!("unexpected {other}"),
    }
}

#[test]
fn fractional_end_frame_rejected_before_dispatch() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = unpublished_graph_with_time(fractional_time(
        TimePoint::frames(1, 30, 1).unwrap(),
        TimePoint::frames_rational(5, 2, 30, 1).unwrap(),
    ));
    let err = assemble_for_graph(&mut catalog, &g).unwrap_err();
    match err {
        AppError::Orchestration(msg) => {
            assert!(msg.contains("unsupported execution-time provenance"), "{msg}");
        }
        other => panic!("unexpected {other}"),
    }
}

#[test]
fn integral_frame_motion_unchanged() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = unpublished_graph();
    let request = assemble_for_graph(&mut catalog, &g).unwrap();
    assert_eq!(request.frame_start(), 1);
    assert_eq!(request.frame_end(), 61);
    assert_eq!(request.fps_num(), 30);
    assert_eq!(request.fps_den(), 1);
    assert_eq!(g.motion_version.time().start().value_den(), 1);
    assert_eq!(g.motion_version.time().end().value_den(), 1);
}

#[test]
fn validated_motion_json_keeps_rational_frame_points() {
    let g = unpublished_graph();
    let mut value = serde_json::to_value(&g.motion_version).unwrap();
    value["time"]["start"]["value_num"] = serde_json::json!(3);
    value["time"]["start"]["value_den"] = serde_json::json!(2);
    value["time"]["end"]["value_num"] = serde_json::json!(4);
    value["time"]["end"]["value_den"] = serde_json::json!(1);
    let loaded = from_json_validated::<MotionAssetVersion>(&value.to_string()).unwrap();
    assert_eq!(loaded.time().start().value_num(), 3);
    assert_eq!(loaded.time().start().value_den(), 2);
    assert!(!loaded.time().start().is_integral_frame());
}

#[test]
fn fractional_frame_never_reaches_fake_worker() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = unpublished_graph_with_time(fractional_time(
        TimePoint::frames_rational(3, 2, 30, 1).unwrap(),
        TimePoint::frames(4, 30, 1).unwrap(),
    ));
    let spec = seed_graph(&mut catalog, &g);
    let run = catalog.enqueue_job(spec).unwrap();
    catalog.mark_dispatchable(&run.run_id).unwrap();
    let mut worker = FakeWorker::default();
    let err = catalog.dispatch(&run.run_id, &mut worker).unwrap_err();
    assert!(
        err.to_string().contains("unsupported execution-time provenance"),
        "{err}"
    );
    assert!(worker.last_attempt_id.is_none());
    assert_eq!(
        catalog.load_job_run(&run.run_id).unwrap().state,
        rigforge_app::JobRunState::Failed
    );
}
