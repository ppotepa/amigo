import type React from "react";
import type { EditorEvent } from "./editorEvents";
import type { EditorTask } from "./editorTasks";

type TaskAction =
  | { type: "taskStarted"; task: EditorTask }
  | { type: "taskFinished"; taskId: string }
  | { type: "taskFailed"; taskId: string; error: string };

export async function runEditorTask<T>({
  completed,
  dispatch,
  emit,
  failed,
  requested,
  run,
  started,
  task,
}: {
  completed: (result: T) => EditorEvent;
  dispatch: React.Dispatch<TaskAction>;
  emit: (event: EditorEvent) => void;
  failed: (error: string) => EditorEvent;
  requested?: EditorEvent;
  run: () => Promise<T>;
  started?: EditorEvent;
  task: EditorTask;
}): Promise<T | null> {
  if (requested) {
    emit(requested);
  }

  dispatch({ type: "taskStarted", task });

  if (started) {
    emit(started);
  }

  try {
    const result = await run();
    dispatch({ type: "taskFinished", taskId: task.id });
    emit(completed(result));
    return result;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    dispatch({ type: "taskFailed", taskId: task.id, error: message });
    emit(failed(message));
    return null;
  }
}
