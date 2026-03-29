import { formatBytes } from "../lib/types";

interface Props {
  claimed: number;
  real: number;
}

export default function CapacityBar({ claimed, real }: Props) {
  const ratio = claimed > 0 ? real / claimed : 0;
  const percent = Math.round(ratio * 100);
  const isFake = ratio < 0.9;

  return (
    <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm">
      <h3 className="text-lg font-semibold text-gray-800 dark:text-gray-200 mb-4">
        容量验证
      </h3>

      <div className="flex items-end gap-6 mb-4">
        <div className="text-center">
          <div className="text-3xl font-bold text-gray-900 dark:text-gray-100">
            {formatBytes(claimed)}
          </div>
          <div className="text-sm text-gray-500">标称容量</div>
        </div>
        <div className="text-2xl text-gray-400">vs</div>
        <div className="text-center">
          <div
            className={`text-3xl font-bold ${isFake ? "text-red-500" : "text-green-500"}`}
          >
            {formatBytes(real)}
          </div>
          <div className="text-sm text-gray-500">真实容量</div>
        </div>
      </div>

      <div className="relative">
        <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-6">
          <div
            className={`h-6 rounded-full transition-all flex items-center justify-center text-xs font-medium text-white ${
              isFake ? "bg-red-500" : "bg-green-500"
            }`}
            style={{ width: `${Math.min(percent, 100)}%` }}
          >
            {percent}%
          </div>
        </div>
      </div>

      {isFake && (
        <div className="mt-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-700 dark:text-red-300 px-4 py-2 rounded-lg text-sm">
          警告：真实容量仅为标称的 {percent}%，疑似扩容盘！
        </div>
      )}
    </div>
  );
}
