import { useNavigate } from "react-router-dom";
import { useDrives } from "../hooks/useDrives";
import DriveCard from "../components/DriveCard";

export default function Home() {
  const { drives, loading, error, refresh } = useDrives();
  const navigate = useNavigate();

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            U盘设备列表
          </h1>
          <p className="text-gray-500 dark:text-gray-400 mt-1">
            检测到 {drives.length} 个可移动存储设备
          </p>
        </div>
        <button
          onClick={refresh}
          disabled={loading}
          className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 transition-colors text-sm font-medium"
        >
          {loading ? "刷新中..." : "刷新设备"}
        </button>
      </div>

      {error && (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-700 dark:text-red-300 px-4 py-3 rounded-lg mb-4">
          {error}
        </div>
      )}

      {drives.length === 0 && !loading ? (
        <div className="text-center py-20">
          <div className="text-6xl mb-4 opacity-30">U</div>
          <p className="text-gray-500 dark:text-gray-400 text-lg">
            未检测到U盘设备
          </p>
          <p className="text-gray-400 dark:text-gray-500 mt-2">
            请插入U盘后点击刷新
          </p>
        </div>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {drives.map((drive) => (
            <DriveCard
              key={drive.path}
              drive={drive}
              onTest={() =>
                navigate("/test", {
                  state: { drive },
                })
              }
            />
          ))}
        </div>
      )}
    </div>
  );
}
