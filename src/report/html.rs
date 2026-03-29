pub fn generate_html_report(
    drive_name: &str,
    test_date: &str,
    claimed_capacity_gb: f64,
    real_capacity_gb: Option<f64>,
    seq_read: Option<f64>,
    seq_write: Option<f64>,
    random_read_iops: Option<f64>,
    random_write_iops: Option<f64>,
    stability: Option<f64>,
    bad_blocks: u64,
    _total_blocks: u64,
    total_score: u32,
    speed_samples_json: &str,
    bad_block_positions_json: &str,
) -> String {
    let score_class = if total_score >= 90 {
        "excellent"
    } else if total_score >= 70 {
        "good"
    } else if total_score >= 50 {
        "fair"
    } else {
        "poor"
    };

    let grade_text = if total_score >= 90 {
        "优秀"
    } else if total_score >= 70 {
        "良好"
    } else if total_score >= 50 {
        "一般"
    } else {
        "差"
    };

    let real_gb = match real_capacity_gb {
        Some(v) => format!("{:.1} GB", v),
        None => "未测试".to_string(),
    };

    let seq_read_str = match seq_read {
        Some(v) => format!("{:.1} MB/s", v),
        None => "N/A".to_string(),
    };

    let seq_write_str = match seq_write {
        Some(v) => format!("{:.1} MB/s", v),
        None => "N/A".to_string(),
    };

    let rand_read_str = match random_read_iops {
        Some(v) => format!("{:.0}", v),
        None => "N/A".to_string(),
    };

    let rand_write_str = match random_write_iops {
        Some(v) => format!("{:.0}", v),
        None => "N/A".to_string(),
    };

    let stability_str = match stability {
        Some(v) => format!("{:.0}%", v * 100.0),
        None => "N/A".to_string(),
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<title>UDisk Inspector 检测报告 - {drive_name}</title>
<script src="https://cdn.jsdelivr.net/npm/echarts@5/dist/echarts.min.js"></script>
<style>
  body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; max-width: 900px; margin: 0 auto; padding: 20px; background: #f5f5f5; }}
  .header {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 30px; border-radius: 12px; margin-bottom: 20px; }}
  .header h1 {{ margin: 0 0 8px 0; font-size: 24px; }}
  .header p {{ margin: 0; opacity: 0.9; }}
  .card {{ background: white; border-radius: 12px; padding: 24px; margin-bottom: 16px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }}
  .card h2 {{ margin-top: 0; color: #333; font-size: 18px; border-bottom: 2px solid #eee; padding-bottom: 8px; }}
  .score {{ font-size: 64px; font-weight: bold; text-align: center; }}
  .score.excellent {{ color: #22c55e; }}
  .score.good {{ color: #3b82f6; }}
  .score.fair {{ color: #eab308; }}
  .score.poor {{ color: #ef4444; }}
  .stat-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 16px; }}
  .stat {{ text-align: center; padding: 16px; background: #f8fafc; border-radius: 8px; }}
  .stat-value {{ font-size: 28px; font-weight: bold; color: #1e40af; }}
  .stat-label {{ font-size: 14px; color: #64748b; margin-top: 4px; }}
  .chart {{ width: 100%; height: 300px; }}
  .grade-badge {{ display: inline-block; padding: 4px 12px; border-radius: 20px; font-size: 14px; font-weight: 600; }}
  .grade-excellent {{ background: #dcfce7; color: #166534; }}
  .grade-good {{ background: #dbeafe; color: #1e40af; }}
  .grade-fair {{ background: #fef3c7; color: #92400e; }}
  .grade-poor {{ background: #fee2e2; color: #991b1b; }}
</style>
</head>
<body>
<div class="header">
  <h1>U盘质量检测报告</h1>
  <p>{drive_name} | 检测日期: {test_date}</p>
</div>

<div class="card">
  <h2>综合评分</h2>
  <div class="score {score_class}">{total_score}<span style="font-size:24px">/100</span></div>
  <div style="text-align:center"><span class="grade-badge grade-{score_class}">{grade_text}</span></div>
</div>

<div class="card">
  <h2>检测数据</h2>
  <div class="stat-grid">
    <div class="stat">
      <div class="stat-value">{claimed_gb:.1} GB</div>
      <div class="stat-label">标称容量</div>
    </div>
    <div class="stat">
      <div class="stat-value">{real_gb}</div>
      <div class="stat-label">真实容量</div>
    </div>
    <div class="stat">
      <div class="stat-value">{seq_read_str}</div>
      <div class="stat-label">顺序读取</div>
    </div>
    <div class="stat">
      <div class="stat-value">{seq_write_str}</div>
      <div class="stat-label">顺序写入</div>
    </div>
    <div class="stat">
      <div class="stat-value">{rand_read_str}</div>
      <div class="stat-label">随机读取 IOPS</div>
    </div>
    <div class="stat">
      <div class="stat-value">{rand_write_str}</div>
      <div class="stat-label">随机写入 IOPS</div>
    </div>
    <div class="stat">
      <div class="stat-value">{stability_str}</div>
      <div class="stat-label">速度稳定性</div>
    </div>
    <div class="stat">
      <div class="stat-value">{bad_blocks}</div>
      <div class="stat-label">坏块数量</div>
    </div>
  </div>
</div>

<div class="card">
  <h2>读写速度曲线</h2>
  <div id="speedChart" class="chart"></div>
</div>

<div class="card">
  <h2>坏块分布</h2>
  <div id="badBlockChart" class="chart"></div>
</div>

<script>
var speedData = {speed_samples_json};
if (speedData && speedData.length > 0) {{
  var chart = echarts.init(document.getElementById('speedChart'));
  chart.setOption({{
    tooltip: {{ trigger: 'axis' }},
    legend: {{ data: ['写入速度', '读取速度'] }},
    xAxis: {{ type: 'category', data: speedData.map(s => s.offset_mb + ' MB'), name: '偏移量' }},
    yAxis: {{ type: 'value', name: 'MB/s' }},
    series: [
      {{ name: '写入速度', type: 'line', data: speedData.map(s => s.write_speed), smooth: true, itemStyle: {{ color: '#ef4444' }} }},
      {{ name: '读取速度', type: 'line', data: speedData.map(s => s.read_speed), smooth: true, itemStyle: {{ color: '#3b82f6' }} }}
    ]
  }});
}}

var badData = {bad_block_positions_json};
if (badData && badData.length > 0) {{
  var chart2 = echarts.init(document.getElementById('badBlockChart'));
  var heatData = badData.map((pos, i) => [pos % 100, Math.floor(pos / 100), 1]);
  chart2.setOption({{
    tooltip: {{}},
    xAxis: {{ type: 'category', splitArea: {{ show: true }} }},
    yAxis: {{ type: 'category', splitArea: {{ show: true }} }},
    visualMap: {{ min: 0, max: 1, calculable: false, orient: 'horizontal', left: 'center', bottom: 0, inRange: {{ color: ['#22c55e', '#ef4444'] }} }},
    series: [{{ type: 'heatmap', data: heatData, label: {{ show: false }} }}]
  }});
}} else {{
  document.getElementById('badBlockChart').innerHTML = '<p style="text-align:center;color:#22c55e;font-size:18px;padding:40px">未发现坏块</p>';
}}
</script>
<div style="text-align:center;color:#94a3b8;font-size:12px;margin-top:20px">
  Generated by UDisk Inspector | {test_date}
</div>
</body>
</html>"#,
        drive_name = drive_name,
        test_date = test_date,
        total_score = total_score,
        score_class = score_class,
        grade_text = grade_text,
        claimed_gb = claimed_capacity_gb,
        real_gb = real_gb,
        seq_read_str = seq_read_str,
        seq_write_str = seq_write_str,
        rand_read_str = rand_read_str,
        rand_write_str = rand_write_str,
        stability_str = stability_str,
        bad_blocks = bad_blocks,
        speed_samples_json = speed_samples_json,
        bad_block_positions_json = bad_block_positions_json,
    )
}
