import ReactECharts from "echarts-for-react";
import type { SpeedSample } from "../lib/types";

interface Props {
  samples: SpeedSample[];
}

export default function SpeedChart({ samples }: Props) {
  const option = {
    title: {
      text: "读写速度曲线",
      textStyle: { fontSize: 16, fontWeight: 600 },
    },
    tooltip: {
      trigger: "axis",
      formatter: (params: Array<{ seriesName: string; value: number; axisValue: string }>) => {
        let tip = `${params[0].axisValue}<br/>`;
        for (const p of params) {
          tip += `${p.seriesName}: ${p.value.toFixed(1)} MB/s<br/>`;
        }
        return tip;
      },
    },
    legend: {
      data: ["写入速度", "读取速度"],
      bottom: 0,
    },
    grid: {
      left: "3%",
      right: "4%",
      bottom: "12%",
      containLabel: true,
    },
    xAxis: {
      type: "category",
      data: samples.map((s) => `${s.offset_mb} MB`),
      name: "偏移量",
      axisLabel: {
        rotate: samples.length > 20 ? 45 : 0,
      },
    },
    yAxis: {
      type: "value",
      name: "MB/s",
    },
    series: [
      {
        name: "写入速度",
        type: "line",
        data: samples.map((s) => s.write_speed),
        smooth: true,
        itemStyle: { color: "#ef4444" },
        areaStyle: { color: "rgba(239, 68, 68, 0.1)" },
      },
      {
        name: "读取速度",
        type: "line",
        data: samples.map((s) => s.read_speed),
        smooth: true,
        itemStyle: { color: "#3b82f6" },
        areaStyle: { color: "rgba(59, 130, 246, 0.1)" },
      },
    ],
  };

  return (
    <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm">
      <ReactECharts option={option} style={{ height: 350 }} />
    </div>
  );
}
