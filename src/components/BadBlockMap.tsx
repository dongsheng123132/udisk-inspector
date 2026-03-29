import ReactECharts from "echarts-for-react";

interface Props {
  badBlocks: number[];
  totalBlocks: number;
}

export default function BadBlockMap({ badBlocks, totalBlocks }: Props) {
  if (badBlocks.length === 0) {
    return (
      <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm">
        <h3 className="text-lg font-semibold text-gray-800 dark:text-gray-200 mb-4">
          坏块分布
        </h3>
        <div className="text-center py-8">
          <div className="text-4xl font-bold text-green-500 mb-2">0</div>
          <p className="text-gray-500">未发现坏块</p>
        </div>
      </div>
    );
  }

  const cols = Math.min(100, Math.ceil(Math.sqrt(totalBlocks)));
  const badSet = new Set(badBlocks);
  const data: [number, number, number][] = [];

  for (let i = 0; i < totalBlocks; i++) {
    if (badSet.has(i)) {
      data.push([i % cols, Math.floor(i / cols), 1]);
    }
  }

  const option = {
    title: {
      text: `坏块分布 (${badBlocks.length} 个坏块)`,
      textStyle: { fontSize: 16, fontWeight: 600 },
    },
    tooltip: {
      formatter: (params: { value: [number, number, number] }) => {
        const blockNum = params.value[1] * cols + params.value[0];
        return `块 #${blockNum} - 坏块`;
      },
    },
    xAxis: { type: "category", show: false },
    yAxis: { type: "category", show: false },
    visualMap: {
      show: false,
      min: 0,
      max: 1,
      inRange: { color: ["#22c55e", "#ef4444"] },
    },
    series: [
      {
        type: "heatmap",
        data,
        label: { show: false },
        itemStyle: { borderColor: "#fff", borderWidth: 1 },
      },
    ],
  };

  return (
    <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm">
      <ReactECharts option={option} style={{ height: 300 }} />
    </div>
  );
}
