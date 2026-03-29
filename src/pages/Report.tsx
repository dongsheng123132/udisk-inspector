import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import type { ReportDetail } from "../lib/types";
import { getReport, exportHtml } from "../lib/tauri";
import ScoreGauge from "../components/ScoreGauge";
import CapacityBar from "../components/CapacityBar";
import SpeedChart from "../components/SpeedChart";
import BadBlockMap from "../components/BadBlockMap";
import StabilityIndicator from "../components/StabilityIndicator";
import { formatBytes } from "../lib/types";

export default function Report() {
  const { id } = useParams<{ id: string }>();
  const [report, setReport] = useState<ReportDetail | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!id) return;
    getReport(id)
      .then(setReport)
      .catch((e) => setError(String(e)));
  }, [id]);

  const handleExport = async () => {
    if (!id) return;
    try {
      const html = await exportHtml(id);
      const blob = new Blob([html], { type: "text/html" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `udisk-report-${id}.html`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      setError(String(e));
    }
  };

  if (error) {
    return (
      <div className="bg-red-50 text-red-700 px-4 py-3 rounded-lg">
        {error}
      </div>
    );
  }

  if (!report) {
    return <div className="text-gray-500 text-center py-20">加载中...</div>;
  }

  const speedSamples = report.details_json
    ? JSON.parse(report.details_json)
    : [];

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            检测报告
          </h1>
          <p className="text-gray-500 mt-1">
            {report.drive_name} | {report.test_date}
          </p>
        </div>
        <button
          onClick={handleExport}
          className="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 text-sm font-medium"
        >
          导出HTML报告
        </button>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <ScoreGauge
          score={report.total_score}
          capacityScore={report.capacity_score}
          speedScore={report.speed_score}
          stabilityScore={report.stability_score}
          badblockScore={report.badblock_score}
        />

        <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm">
          <h2 className="text-lg font-semibold text-gray-800 dark:text-gray-200 mb-4">
            设备信息
          </h2>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-500">设备名称</span>
              <span className="text-gray-900 dark:text-gray-100 font-medium">
                {report.drive_name}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-500">序列号</span>
              <span className="text-gray-900 dark:text-gray-100 font-mono">
                {report.drive_serial || "N/A"}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-500">标称容量</span>
              <span className="text-gray-900 dark:text-gray-100">
                {formatBytes(report.claimed_capacity_bytes)}
              </span>
            </div>
            {report.real_capacity_bytes != null && (
              <div className="flex justify-between">
                <span className="text-gray-500">真实容量</span>
                <span className="text-gray-900 dark:text-gray-100">
                  {formatBytes(report.real_capacity_bytes)}
                </span>
              </div>
            )}
            {report.seq_read_speed != null && (
              <div className="flex justify-between">
                <span className="text-gray-500">顺序读取</span>
                <span className="text-gray-900 dark:text-gray-100">
                  {report.seq_read_speed.toFixed(1)} MB/s
                </span>
              </div>
            )}
            {report.seq_write_speed != null && (
              <div className="flex justify-between">
                <span className="text-gray-500">顺序写入</span>
                <span className="text-gray-900 dark:text-gray-100">
                  {report.seq_write_speed.toFixed(1)} MB/s
                </span>
              </div>
            )}
            {report.test_duration_secs != null && (
              <div className="flex justify-between">
                <span className="text-gray-500">测试用时</span>
                <span className="text-gray-900 dark:text-gray-100">
                  {Math.floor(report.test_duration_secs / 60)}分
                  {report.test_duration_secs % 60}秒
                </span>
              </div>
            )}
          </div>
        </div>
      </div>

      {report.real_capacity_bytes != null && (
        <div className="mt-4">
          <CapacityBar
            claimed={report.claimed_capacity_bytes}
            real={report.real_capacity_bytes}
          />
        </div>
      )}

      {speedSamples.length > 0 && (
        <div className="mt-4">
          <SpeedChart samples={speedSamples} />
        </div>
      )}

      {report.speed_stability != null && (
        <div className="mt-4">
          <StabilityIndicator stability={report.speed_stability} speedDrops={0} />
        </div>
      )}

      {report.bad_block_count != null && report.bad_block_count > 0 && (
        <div className="mt-4">
          <BadBlockMap badBlocks={[]} totalBlocks={report.total_blocks || 0} />
        </div>
      )}
    </div>
  );
}
