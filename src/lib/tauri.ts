import { invoke } from "@tauri-apps/api/core";
import type { DriveInfo, TestConfig, ReportSummary, ReportDetail } from "./types";

export async function listDrives(): Promise<DriveInfo[]> {
  return invoke<DriveInfo[]>("list_drives");
}

export async function getDriveInfo(path: string): Promise<DriveInfo> {
  return invoke<DriveInfo>("get_drive_info", { path });
}

export async function startTest(config: TestConfig): Promise<string> {
  return invoke<string>("start_test", { config });
}

export async function stopTest(): Promise<void> {
  return invoke<void>("stop_test");
}

export async function getReport(id: string): Promise<ReportDetail> {
  return invoke<ReportDetail>("get_report", { id });
}

export async function listReports(): Promise<ReportSummary[]> {
  return invoke<ReportSummary[]>("list_reports");
}

export async function exportHtml(id: string): Promise<string> {
  return invoke<string>("export_html", { id });
}

export async function deleteReport(id: string): Promise<void> {
  return invoke<void>("delete_report", { id });
}
