use stratum::postprocessors::vlog_bias::VlogBiasPostprocessor;
use stratum::core::schema::{Node, NodeWithScore};
use stratum::core::query_bundle::QueryBundle;
use stratum::postprocessors::base::NodePostprocessor;

#[tokio::test]
async fn test_vlog_nullification_vulnerability() {
    // 意図的に検索結果を全てゼロにするバイアスを注入
    let vlog_content = "@BIAS:{Negative:100.0}";
    let postprocessor = VlogBiasPostprocessor::from_vlog(vlog_content, None);
    
    let nodes = vec![
        NodeWithScore { node: Node::new_text("test".into()), score: Some(1.0) }
    ];
    
    let result = postprocessor.postprocess_nodes(nodes, &QueryBundle::new("query".into())).await.unwrap();
    
    // 現在のスコア計算ロジックでは、 boost が適用され 0 になることはないが、
    // ここで score が NaN にならないか、あるいは予期せぬソート順にならないかを検証する。
    assert!(result[0].score.unwrap() >= 0.0);
}
