import type { DriveInfo } from "../lib/types";
import { formatBytes } from "../lib/types";

interface Props {
  drive: DriveInfo;
  onTest: () => void;
}

export default function DriveCard({ drive, onTest }: Props) {
  const usedBytes = drive.capacity_bytes - drive.free_bytes;
  const usedPercent =
    drive.capacity_bytes > 0
      ? Math.round((usedBytes / drive.capacity_bytes) * 100)
      : 0;

  return (
    <div className="bg-white dark:bg-gray-800 rounded-xl p-5 shadow-sm hover:shadow-md transition-shadow">
      <div className="flex items-start justify-between mb-3">
        <div>
          <h3 className="font-semibold text-gray-900 dark:text-gray-100">
            {drive.name || "未知设备"}
          </h3>
          <p className="text-sm text-gray-500 dark:text-gray-400 mt-0.5">
            {drive.path} | {drive.file_system}
          </p>
        </div>
        <span className="px-2 py-1 bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-300 text-xs rounded-lg font-medium">
          已连接
        </span>
      </div>

      <div className="mb-3">
        <div className="flex justify-between text-sm mb-1">
          <span className="text-gray-500">
            已用 {formatBytes(usedBytes)}
          </span>
          <span className="text-gray-500">
            {formatBytes(drive.capacity_bytes)}
          </span>
        </div>
        <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2.5">
          <div
            className="bg-blue-600 h-2.5 rounded-full transition-all"
            style={{ width: `${usedPercent}%` }}
          />
        </div>
        <p className="text-xs text-gray-400 mt-1">
          可用 {formatBytes(drive.free_bytes)} ({100 - usedPercent}%)
        </p>
      </div>

      <button
        onClick={onTest}
        className="w-full py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors text-sm font-medium"
      >
        开始检测
      </button>
    </div>
  );
}
