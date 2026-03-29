interface Props {
  stability: number; // 0.0 - 1.0
  speedDrops: number;
}

export default function StabilityIndicator({ stability, speedDrops }: Props) {
  const percent = Math.round(stability * 100);

  let riskLevel: string;
  let riskColor: string;
  let riskBg: string;
  let description: string;

  if (stability >= 0.85 && speedDrops <= 1) {
    riskLevel = "低风险";
    riskColor = "text-green-600";
    riskBg = "bg-green-50 dark:bg-green-900/20";
    description = "速度稳定，发热风险低。该U盘主控和闪存质量较好。";
  } else if (stability >= 0.6 && speedDrops <= 3) {
    riskLevel = "中等风险";
    riskColor = "text-yellow-600";
    riskBg = "bg-yellow-50 dark:bg-yellow-900/20";
    description =
      "速度有一定波动，可能在长时间使用后出现明显发热。建议避免连续大量写入。";
  } else {
    riskLevel = "高风险";
    riskColor = "text-red-600";
    riskBg = "bg-red-50 dark:bg-red-900/20";
    description =
      "速度极不稳定，存在频繁骤降。该U盘可能有严重发热问题，主控或闪存质量较差。";
  }

  return (
    <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm">
      <h3 className="text-lg font-semibold text-gray-800 dark:text-gray-200 mb-4">
        速度稳定性 & 发热风险
      </h3>

      <div className="flex items-center gap-6 mb-4">
        <div className="text-center">
          <div className="text-4xl font-bold text-gray-900 dark:text-gray-100">
            {percent}%
          </div>
          <div className="text-sm text-gray-500">稳定性指数</div>
        </div>
        <div className="text-center">
          <div className="text-4xl font-bold text-gray-900 dark:text-gray-100">
            {speedDrops}
          </div>
          <div className="text-sm text-gray-500">速度骤降次数</div>
        </div>
        <div className="text-center">
          <div className={`text-2xl font-bold ${riskColor}`}>{riskLevel}</div>
          <div className="text-sm text-gray-500">发热风险</div>
        </div>
      </div>

      <div className={`${riskBg} rounded-lg p-4`}>
        <p className={`text-sm ${riskColor}`}>{description}</p>
      </div>
    </div>
  );
}
