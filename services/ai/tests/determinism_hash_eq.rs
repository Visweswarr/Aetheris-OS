use aetheris_ai::aicore::CognitiveCore;
use aetheris_ai::aicore::CognitiveCoreConfig;

#[tokio::test]
async fn snapshot_id_stable_for_same_state() {
    std::env::set_var("NGFS_ROOT", "./target/test-ngfs");
    std::env::set_var("AICORE_DET_SEED", "42");
    let core = CognitiveCore::new(CognitiveCoreConfig::default()).await.unwrap();
    // Submit a deterministic goal (RealIntentBus may be stubbed depending on INTENT_SOCKET)
    let _ = core.submit_goal("determinism").await;
    let id1 = core.snapshot_write().await.unwrap();
    let id2 = core.snapshot_write().await.unwrap();
    assert_eq!(id1, id2);
    core.snapshot_replay(&id1).await.unwrap();
}
