import { useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import type { DriveInfo, TestConfig } from "../lib/types";
import { useTest } from "../hooks/useTest";
import { useDrives } from "../hooks/useDrives";
import TestProgress from "../components/TestProgress";
import SpeedChart from "../components/SpeedChart";
import ScoreGauge from "../components/ScoreGauge";
import CapacityBar from "../components/CapacityBar";
import BadBlockMap from "../components/BadBlockMap";
import StabilityIndicator from "../components/StabilityIndicator";

export default function Test() {
  const location = useLocation();
  const navigate = useNavigate();
  const { drives } = useDrives();
  const { running, progress, result, error, start, stop } = useTest();

  const initialDrive = (location.state as { drive?: DriveInfo })?.drive;
  const [selectedDrive, setSelectedDrive] = useState<DriveInfo | null>(
    initialDrive || null,
  );
  const [testCapacity, setTestCapacity] = useState(true);
  const [testSpeed, setTestSpeed] = useState(true);
  const [testBadblock, setTestBadblock] = useState(true);
  const [destructive, setDestructive] = useState(false);
  const [testSizeMb, setTestSizeMb] = useState(1024);

  const handleStart = async () => {
    if (!selectedDrive) return;
    const config: TestConfig = {
      drive_path: selectedDrive.path,
      mount_point: selectedDrive.mount_point,
      test_capacity: testCapacity,
      test_speed: testSpeed,
      test_badblock: testBadblock,
      destructive,
      test_size_mb: testSizeMb,
    };
    await start(config);
  };

  return (
    <div>
      <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-6">
        U盘质量检测
      </h1>

      {!running && !result && (
        <div className="space-y-4">
          {/* Drive selection */}
          <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm">
            <h2 className="text-lg font-semibold text-gray-800 dark:text-gray-200 mb-4">
              选择设备
            </h2>
            <div className="grid gap-3 md:grid-cols-2 lg:grid-cols-3">
              {drives.map((d) => (
                <button
                  key={d.path}
                  onClick={() => setSelectedDrive(d)}
                  className={`text-left p-4 rounded-lg border-2 transition-all ${
                    selectedDrive?.path === d.path
                      ? "border-blue-500 bg-blue-50 dark:bg-blue-900/20"
                      : "border-gray-200 dark:border-gray-600 hover:border-gray-300"
                  }`}
                >
                  <div className="font-medium text-gray-900 dark:text-gray-100">
                    {d.name || d.path}
                  </div>
                  <div className="text-sm text-gray-500 dark:text-gray-400">
                    {d.path} | {(d.capacity_bytes / 1e9).toFixed(1)} GB |{" "}
                    {d.file_system}
                  </div>
                </button>
              ))}
              {drives.length === 0 && (
                <p className="text-gray-400 col-span-full">未检测到U盘设备</p>
              )}
            </div>
          </div>

          {/* Test options */}
          <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm">
            <h2 className="text-lg font-semibold text-gray-800 dark:text-gray-200 mb-4">
              测试项目
            </h2>
            <div className="space-y-3">
              <label className="flex items-center gap-3 cursor-pointer">
                <input
                  type="checkbox"
                  checked={testCapacity}
                  onChange={(e) => setTestCapacity(e.target.checked)}
                  className="w-5 h-5 text-blue-600 rounded"
                />
                <div>
                  <span className="font-medium text-gray-800 dark:text-gray-200">
                    扩容检测
                  </span>
                  <span className="text-sm text-gray-500 ml-2">
                    验证U盘真实容量，检测假冒大容量
                  </span>
                </div>
              </label>
              <label className="flex items-center gap-3 cursor-pointer">
                <input
                  type="checkbox"
                  checked={testSpeed}
                  onChange={(e) => setTestSpeed(e.target.checked)}
                  className="w-5 h-5 text-blue-600 rounded"
                />
                <div>
                  <span className="font-medium text-gray-800 dark:text-gray-200">
                    速度测试
                  </span>
                  <span className="text-sm text-gray-500 ml-2">
                    顺序读写 + 随机4K + 稳定性分析
                  </span>
                </div>
              </label>
              {testSpeed && (
                <div className="ml-8">
                  <label className="text-sm text-gray-600 dark:text-gray-400">
                    测试数据量:
                    <select
                      value={testSizeMb}
                      onChange={(e) => setTestSizeMb(Number(e.target.value))}
                      className="ml-2 rounded border-gray-300 dark:border-gray-600 dark:bg-gray-700 text-sm"
                    >
                      <option value={256}>256 MB</option>
                      <option value={512}>512 MB</option>
                      <option value={1024}>1 GB</option>
                      <option value={2048}>2 GB</option>
                      <option value={4096}>4 GB</option>
                    </select>
                  </label>
                </div>
              )}
              <label className="flex items-center gap-3 cursor-pointer">
                <input
                  type="checkbox"
                  checked={testBadblock}
                  onChange={(e) => setTestBadblock(e.target.checked)}
                  className="w-5 h-5 text-blue-600 rounded"
                />
                <div>
                  <span className="font-medium text-gray-800 dark:text-gray-200">
                    坏块扫描
                  </span>
                  <span className="text-sm text-gray-500 ml-2">
                    逐块写入验证，检测存储坏块
                  </span>
                </div>
              </label>
              <div className="pt-2 border-t border-gray-200 dark:border-gray-600">
                <label className="flex items-center gap-3 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={destructive}
                    onChange={(e) => setDestructive(e.target.checked)}
                    className="w-5 h-5 text-red-600 rounded"
                  />
                  <div>
                    <span className="font-medium text-red-600">
                      完整模式（破坏性）
                    </span>
                    <span className="text-sm text-red-400 ml-2">
                      会覆盖U盘全部数据！请先备份
                    </span>
                  </div>
                </label>
              </div>
            </div>
          </div>

          {/* Start button */}
          <button
            onClick={handleStart}
            disabled={!selectedDrive || (!testCapacity && !testSpeed && !testBadblock)}
            className="w-full py-3 bg-gradient-to-r from-blue-600 to-purple-600 text-white rounded-xl font-semibold text-lg hover:from-blue-700 hover:to-purple-700 disabled:opacity-50 disabled:cursor-not-allowed transition-all shadow-lg hover:shadow-xl"
          >
            开始检测
          </button>
        </div>
      )}

      {/* Progress */}
      {running && progress && (
        <div className="space-y-4">
          <TestProgress progress={progress} onStop={stop} />
        </div>
      )}

      {/* Error */}
      {error && (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-700 dark:text-red-300 px-4 py-3 rounded-lg mt-4">
          {error}
        </div>
      )}

      {/* Results */}
      {result && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h2 className="text-xl font-bold text-gray-900 dark:text-gray-100">
              检测结果
            </h2>
            <button
              onClick={() => navigate(`/report/${result.report_id}`)}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 text-sm font-medium"
            >
              查看完整报告
            </button>
          </div>

          <div className="grid gap-4 md:grid-cols-2">
            <ScoreGauge
              score={result.total_score}
              capacityScore={result.capacity_score}
              speedScore={result.speed_score}
              stabilityScore={result.stability_score}
              badblockScore={result.badblock_score}
            />
            {result.real_capacity != null && (
              <CapacityBar
                claimed={result.claimed_capacity}
                real={result.real_capacity}
              />
            )}
          </div>

          {result.speed_samples.length > 0 && (
            <SpeedChart samples={result.speed_samples} />
          )}

          {result.speed_stability != null && (
            <StabilityIndicator
              stability={result.speed_stability}
              speedDrops={0}
            />
          )}

          {result.bad_block_positions.length > 0 && (
            <BadBlockMap
              badBlocks={result.bad_block_positions}
              totalBlocks={result.total_blocks || 0}
            />
          )}
        </div>
      )}
    </div>
  );
}
