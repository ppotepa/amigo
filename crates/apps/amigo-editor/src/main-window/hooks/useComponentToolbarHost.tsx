import { useCallback, useState } from "react";
import type React from "react";
import { ComponentToolbar, defaultToolbarState } from "../../editor-components/ComponentToolbar";
import { editorComponentById } from "../../editor-components/componentRegistry";
import type {
  ComponentToolbarState,
  ComponentToolbarValue,
  EditorComponentInstance,
} from "../../editor-components/componentTypes";
import { toneForComponentDomain } from "../../theme/semanticColorRegistry";

export function useComponentToolbarHost({
  modId,
  refreshProjectTree,
}: {
  modId?: string | null;
  refreshProjectTree: (modId: string) => Promise<void>;
}): {
  renderComponentToolbar: (instance: EditorComponentInstance) => React.ReactNode;
  toolbarStateFor: (instance: EditorComponentInstance) => ComponentToolbarState;
} {
  const [componentToolbarState, setComponentToolbarState] = useState<Record<string, ComponentToolbarState>>({});

  const toolbarStateFor = useCallback(
    (instance: EditorComponentInstance): ComponentToolbarState => {
      const toolbar = editorComponentById(instance.componentId)?.toolbar;
      return {
        ...defaultToolbarState(toolbar),
        ...(componentToolbarState[instance.instanceId] ?? {}),
      };
    },
    [componentToolbarState],
  );

  const setToolbarValue = useCallback(
    (instance: EditorComponentInstance, controlId: string, value: ComponentToolbarValue) => {
      setComponentToolbarState((current) => ({
        ...current,
        [instance.instanceId]: {
          ...defaultToolbarState(editorComponentById(instance.componentId)?.toolbar),
          ...(current[instance.instanceId] ?? {}),
          [controlId]: value,
        },
      }));
    },
    [],
  );

  const runComponentToolbarAction = useCallback(
    (instance: EditorComponentInstance, controlId: string) => {
      if (instance.componentId === "assets.browser" && controlId === "add") {
        setToolbarValue(instance, "addNonce", String(Date.now()));
        return;
      }

      if (instance.componentId === "assets.browser" && controlId === "refresh" && modId) {
        setToolbarValue(instance, "refreshNonce", String(Date.now()));
        void refreshProjectTree(modId);
      }
    },
    [modId, refreshProjectTree, setToolbarValue],
  );

  const renderComponentToolbar = useCallback(
    (instance: EditorComponentInstance) => {
      const definition = editorComponentById(instance.componentId);
      const toolbar = definition?.toolbar;
      if (!toolbar) return null;

      return (
        <ComponentToolbar
          toolbar={toolbar}
          tone={definition ? toneForComponentDomain(definition.domain) : "neutral"}
          state={toolbarStateFor(instance)}
          onChange={(controlId, value) => setToolbarValue(instance, controlId, value)}
          onAction={(controlId) => runComponentToolbarAction(instance, controlId)}
        />
      );
    },
    [runComponentToolbarAction, setToolbarValue, toolbarStateFor],
  );

  return { renderComponentToolbar, toolbarStateFor };
}
