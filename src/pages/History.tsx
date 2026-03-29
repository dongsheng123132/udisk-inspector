import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import type { ReportSummary } from "../lib/types";
import { listReports, deleteReport } from "../lib/tauri";
import { getGrade, getGradeText, getGradeColor } from "../lib/types";

export default function History() {
  const [reports, setReports] = useState<ReportSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const navigate = useNavigate();

  const refresh = async () => {
    setLoading(true);
    try {
      const list = await listReports();
      setReports(list);
    } catch {
      // ignore
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    refresh();
  }, []);

  const handleDelete = async (id: string) => {
    if (!confirm("确定要删除这条报告吗？")) return;
    try {
      await deleteReport(id);
      refresh();
    } catch {
      // ignore
    }
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
          历史报告
        </h1>
        <button
          onClick={refresh}
          className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 text-sm font-medium"
        >
          刷新
        </button>
      </div>

      {loading ? (
        <div className="text-center py-20 text-gray-400">加载中...</div>
      ) : reports.length === 0 ? (
        <div className="text-center py-20">
          <p className="text-gray-400 text-lg">暂无检测报告</p>
          <p className="text-gray-400 mt-2">
            去「开始检测」页面进行U盘质量检测
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {reports.map((r) => {
            const grade = getGrade(r.total_score);
            return (
              <div
                key={r.id}
                className="bg-white dark:bg-gray-800 rounded-xl p-4 shadow-sm flex items-center justify-between hover:shadow-md transition-shadow cursor-pointer"
                onClick={() => navigate(`/report/${r.id}`)}
              >
                <div className="flex items-center gap-4">
                  <div
                    className="w-12 h-12 rounded-full flex items-center justify-center text-white font-bold text-lg"
                    style={{ backgroundColor: getGradeColor(grade) }}
                  >
                    {r.total_score}
                  </div>
                  <div>
                    <div className="font-medium text-gray-900 dark:text-gray-100">
                      {r.drive_name}
                    </div>
                    <div className="text-sm text-gray-500">{r.test_date}</div>
                  </div>
                </div>
                <div className="flex items-center gap-3">
                  <span
                    className="px-3 py-1 rounded-full text-sm font-medium"
                    style={{
                      backgroundColor: getGradeColor(grade) + "20",
                      color: getGradeColor(grade),
                    }}
                  >
                    {getGradeText(grade)}
                  </span>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDelete(r.id);
                    }}
                    className="text-gray-400 hover:text-red-500 transition-colors text-sm"
                  >
                    删除
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
