import type { TestProgress as TP } from "../lib/types";

interface Props {
  progress: TP;
  onStop: () => void;
}

const testTypeLabels: Record<string, string> = {
  capacity: "扩容检测",
  speed: "速度测试",
  badblock: "坏块扫描",
  thermal: "稳定性测试",
};

export default function TestProgress({ progress, onStop }: Props) {
  const percent = Math.round(progress.progress * 100);

  return (
    <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm">
      <div className="flex items-center justify-between mb-4">
        <div>
          <h2 className="text-lg font-semibold text-gray-800 dark:text-gray-200">
            {testTypeLabels[progress.test_type] || progress.test_type}
          </h2>
          <p className="text-sm text-gray-500">{progress.phase}</p>
        </div>
        <button
          onClick={onStop}
          className="px-4 py-2 bg-red-500 text-white rounded-lg hover:bg-red-600 text-sm font-medium"
        >
          停止
        </button>
      </div>

      <div className="mb-3">
        <div className="flex justify-between text-sm mb-1">
          <span className="text-gray-600 dark:text-gray-400">
            {progress.message}
          </span>
          <span className="font-mono text-gray-900 dark:text-gray-100">
            {percent}%
          </span>
        </div>
        <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-3">
          <div
            className="bg-gradient-to-r from-blue-500 to-purple-500 h-3 rounded-full transition-all duration-300"
            style={{ width: `${percent}%` }}
          />
        </div>
      </div>

      {progress.current_speed > 0 && (
        <div className="flex items-center gap-2 text-sm">
          <span className="text-gray-500">当前速度:</span>
          <span className="font-mono text-lg font-bold text-blue-600">
            {progress.current_speed.toFixed(1)}
          </span>
          <span className="text-gray-500">MB/s</span>
        </div>
      )}
    </div>
  );
}
