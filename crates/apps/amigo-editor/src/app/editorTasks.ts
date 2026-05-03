export type TaskStatus = "queued" | "running" | "done" | "failed";
export type BusyLevel = "none" | "local" | "background" | "blocking";

export interface EditorTask {
  id: string;
  label: string;
  status: TaskStatus;
  busyLevel: BusyLevel;
  progress?: number;
  error?: string;
  owner?: string;
  startedAt: number;
  endedAt?: number;
}

export function createTask(id: string, label: string, busyLevel: BusyLevel, owner?: string): EditorTask {
  return {
    id,
    label,
    status: "running",
    busyLevel,
    owner,
    startedAt: Date.now(),
  };
}

export function finishTask(task: EditorTask): EditorTask {
  return {
    ...task,
    status: "done",
    progress: 1,
    endedAt: Date.now(),
  };
}

export function failTask(task: EditorTask, error: string): EditorTask {
  return {
    ...task,
    status: "failed",
    error,
    endedAt: Date.now(),
  };
}
