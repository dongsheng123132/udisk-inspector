import ReactECharts from "echarts-for-react";
import { getGrade, getGradeText, getGradeColor } from "../lib/types";

interface Props {
  score: number;
  capacityScore: number;
  speedScore: number;
  stabilityScore: number;
  badblockScore: number;
}

export default function ScoreGauge({
  score,
  capacityScore,
  speedScore,
  stabilityScore,
  badblockScore,
}: Props) {
  const grade = getGrade(score);
  const color = getGradeColor(grade);

  const option = {
    series: [
      {
        type: "gauge",
        startAngle: 220,
        endAngle: -40,
        min: 0,
        max: 100,
        splitNumber: 10,
        itemStyle: { color },
        progress: {
          show: true,
          roundCap: true,
          width: 18,
        },
        pointer: { show: false },
        axisLine: {
          roundCap: true,
          lineStyle: { width: 18, color: [[1, "#e5e7eb"]] },
        },
        axisTick: { show: false },
        splitLine: { show: false },
        axisLabel: { show: false },
        title: { show: false },
        detail: {
          valueAnimation: true,
          fontSize: 48,
          fontWeight: "bold",
          color,
          offsetCenter: [0, "-10%"],
          formatter: "{value}",
        },
        data: [{ value: score }],
      },
    ],
  };

  const scoreItems = [
    { label: "容量真实性", score: capacityScore, max: 35 },
    { label: "读写速度", score: speedScore, max: 25 },
    { label: "速度稳定性", score: stabilityScore, max: 15 },
    { label: "坏块检测", score: badblockScore, max: 25 },
  ];

  return (
    <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm">
      <ReactECharts option={option} style={{ height: 220 }} />
      <div className="text-center -mt-4 mb-4">
        <span
          className="px-3 py-1 rounded-full text-sm font-semibold"
          style={{
            backgroundColor: color + "20",
            color,
          }}
        >
          {getGradeText(grade)}
        </span>
      </div>
      <div className="space-y-2">
        {scoreItems.map((item) => (
          <div key={item.label} className="flex items-center gap-3">
            <span className="text-sm text-gray-500 w-24 shrink-0">
              {item.label}
            </span>
            <div className="flex-1 bg-gray-200 dark:bg-gray-700 rounded-full h-2">
              <div
                className="h-2 rounded-full transition-all"
                style={{
                  width: `${(item.score / item.max) * 100}%`,
                  backgroundColor: color,
                }}
              />
            </div>
            <span className="text-sm font-mono text-gray-700 dark:text-gray-300 w-12 text-right">
              {item.score}/{item.max}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}
