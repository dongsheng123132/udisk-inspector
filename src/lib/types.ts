export interface DriveInfo {
  name: string;
  path: string;
  mount_point: string;
  serial: string;
  capacity_bytes: number;
  free_bytes: number;
  file_system: string;
  is_removable: boolean;
}

export interface TestConfig {
  drive_path: string;
  mount_point: string;
  test_capacity: boolean;
  test_speed: boolean;
  test_badblock: boolean;
  destructive: boolean;
  test_size_mb?: number;
}

export interface TestProgress {
  test_type: string;
  phase: string;
  progress: number;
  current_speed: number;
  message: string;
  error?: string;
}

export interface SpeedSample {
  offset_mb: number;
  write_speed: number;
  read_speed: number;
}

export interface TestResult {
  report_id: string;
  drive_name: string;
  claimed_capacity: number;
  real_capacity?: number;
  seq_read_speed?: number;
  seq_write_speed?: number;
  random_read_iops?: number;
  random_write_iops?: number;
  speed_stability?: number;
  speed_samples: SpeedSample[];
  bad_block_count?: number;
  total_blocks?: number;
  bad_block_positions: number[];
  total_score: number;
  capacity_score: number;
  speed_score: number;
  stability_score: number;
  badblock_score: number;
  test_duration_secs: number;
}

export interface ReportSummary {
  id: string;
  drive_name: string;
  test_date: string;
  total_score: number;
}

export interface ReportDetail {
  id: string;
  drive_name: string;
  drive_serial: string;
  claimed_capacity_bytes: number;
  test_date: string;
  total_score: number;
  capacity_score: number;
  speed_score: number;
  stability_score: number;
  badblock_score: number;
  real_capacity_bytes?: number;
  seq_read_speed?: number;
  seq_write_speed?: number;
  random_read_iops?: number;
  random_write_iops?: number;
  speed_stability?: number;
  bad_block_count?: number;
  total_blocks?: number;
  test_duration_secs?: number;
  details_json?: string;
}

export type Grade = "excellent" | "good" | "fair" | "poor";

export function getGrade(score: number): Grade {
  if (score >= 90) return "excellent";
  if (score >= 70) return "good";
  if (score >= 50) return "fair";
  return "poor";
}

export function getGradeText(grade: Grade): string {
  const map: Record<Grade, string> = {
    excellent: "优秀",
    good: "良好",
    fair: "一般",
    poor: "差",
  };
  return map[grade];
}

export function getGradeColor(grade: Grade): string {
  const map: Record<Grade, string> = {
    excellent: "#22c55e",
    good: "#3b82f6",
    fair: "#eab308",
    poor: "#ef4444",
  };
  return map[grade];
}

export function formatBytes(bytes: number): string {
  if (bytes >= 1e12) return `${(bytes / 1e12).toFixed(1)} TB`;
  if (bytes >= 1e9) return `${(bytes / 1e9).toFixed(1)} GB`;
  if (bytes >= 1e6) return `${(bytes / 1e6).toFixed(1)} MB`;
  return `${(bytes / 1e3).toFixed(1)} KB`;
}
