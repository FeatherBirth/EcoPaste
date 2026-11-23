import { useMount } from "ahooks";
import { Spin } from "antd";
import { motion } from "motion/react";
import { type FC, useCallback, useEffect, useMemo, useState } from "react";
import { useSnapshot } from "valtio";
import {
  type ClipboardPreviewGesturePayload,
  type ClipboardPreviewState,
  getClipboardPreviewState,
  setClipboardPreviewPanelRect,
} from "@/commands";
import { TAURI_EVENT } from "@/constants/events";
import { WINDOW_LABEL } from "@/constants/windows";
import { useTauriListen } from "@/hooks/useTauriListen";
import { settingsState } from "@/stores/settings";
import { cn } from "@/utils/cn";
import { log } from "@/utils/log";
import { cacheKey } from "./cache";
import { PreviewContent, PreviewHeader } from "./components/PreviewContent";
import PreviewContentTransition from "./components/PreviewContentTransition";
import {
  PREVIEW_CONNECTOR_DOT_RADIUS,
  PREVIEW_CONNECTOR_VARIANTS,
  PREVIEW_IMAGE_ZOOM_MAX,
  PREVIEW_IMAGE_ZOOM_MIN,
  PREVIEW_PANEL_MAX_HEIGHT,
  PREVIEW_PANEL_MAX_WIDTH,
  PREVIEW_PANEL_TRANSITION,
  PREVIEW_PANEL_VARIANTS,
  PREVIEW_TEXT_OVERFLOW_SETTLE_MS,
} from "./constants";
import { resolveConnector } from "./geometry";
import { usePreviewPayload, usePreviewRenderState } from "./hooks";
import {
  rectStyle,
  resolveDynamicPanelRect,
  resolveEffectivePanelSize,
  resolveMeasurePanelStyle,
  resolvePreviewPanelLimits,
} from "./layout";
import { hasMeasuredPanelSize, useMeasuredPanelSize } from "./measurement";
import { usePreviewMotion } from "./motion";

const EMPTY_RECT = {
  height: 1,
  left: 0,
  top: 0,
  width: 1,
};
const EMPTY_POINT = { x: 0, y: 0 };
const DEFAULT_PANEL_LIMITS = {
  height: PREVIEW_PANEL_MAX_HEIGHT,
  width: PREVIEW_PANEL_MAX_WIDTH,
};
const EMPTY_CONNECTOR = {
  control1: EMPTY_POINT,
  control2: EMPTY_POINT,
  path: "M 0 0 C 0 0 0 0 0 0",
  source: EMPTY_POINT,
  sourceDot: EMPTY_POINT,
  sourceSide: "right",
  target: EMPTY_POINT,
  targetDot: EMPTY_POINT,
  targetSide: "left",
} as const;

interface BeforeDestroyPayload {
  label: string;
}

/**
 * 系统级剪贴板预览窗口。
 * 预览窗口自身常驻透明 overlay，按 `itemId + updatedAt` 缓存最近内容并渲染基础 Content Viewer。
 */
const Preview: FC = () => {
  const [previewState, setPreviewState] =
    useState<ClipboardPreviewState | null>(null);
  const [payloadResetToken, setPayloadResetToken] = useState(0);
  const [imageZoomState, setImageZoomState] = useState<{
    sessionId: number | null;
    zoom: number;
  }>({ sessionId: null, zoom: 1 });
  const [textHeightState, setTextHeightState] = useState<{
    itemId: string | null;
    height: number;
  }>({ height: 0, itemId: null });
  const [textOverflowReadyKey, setTextOverflowReadyKey] = useState<
    number | null
  >(null);
  const { clipboard } = useSnapshot(settingsState);
  const redactSecrets = clipboard.sensitive.redactSecrets;
  const renderState = usePreviewRenderState(previewState);
  const { loadingItemId, payload } = usePreviewPayload(
    previewState,
    payloadResetToken,
  );
  const { measureRef: panelMeasureRef, size: measuredPanelSize } =
    useMeasuredPanelSize();
  const active = previewState !== null;
  const visibleState = previewState ?? renderState;
  const visibleItemId = visibleState?.itemId ?? null;
  const visibleSessionId = visibleState?.sessionId ?? null;
  const imageZoom =
    imageZoomState.sessionId === visibleSessionId ? imageZoomState.zoom : 1;
  const textContentHeight =
    textHeightState.itemId === visibleItemId ? textHeightState.height : null;
  const usesFullPreviewViewport =
    payload?.kind === "image" || payload?.singleImagePreview === true;
  const panelLimits = visibleState
    ? resolvePreviewPanelLimits(visibleState, usesFullPreviewViewport)
    : DEFAULT_PANEL_LIMITS;
  const effectivePanelSize = visibleState
    ? resolveEffectivePanelSize(
        visibleState.layout,
        measuredPanelSize,
        payload,
        panelLimits,
        imageZoom,
        textContentHeight,
      )
    : measuredPanelSize;
  const panelRect = visibleState
    ? resolveDynamicPanelRect(
        visibleState.layout,
        effectivePanelSize,
        panelLimits,
      )
    : EMPTY_RECT;
  const reportedPanelRect = useMemo(() => {
    return {
      height: panelRect.height,
      left: panelRect.left,
      top: panelRect.top,
      width: panelRect.width,
    };
  }, [panelRect.height, panelRect.left, panelRect.top, panelRect.width]);
  const previewRequestId = previewState?.requestId ?? null;
  const panelMeasureStyle = visibleState
    ? resolveMeasurePanelStyle(visibleState.layout)
    : void 0;
  const connector = visibleState
    ? resolveConnector(
        visibleState.layout.sourceRect,
        panelRect,
        visibleState.layout.placement,
      )
    : EMPTY_CONNECTOR;
  const panelLayoutReady =
    payload?.id === visibleState?.itemId &&
    hasMeasuredPanelSize(effectivePanelSize);
  const motionLayout = usePreviewMotion(
    active,
    visibleState?.sessionId ?? null,
    panelLayoutReady,
    panelRect,
    connector,
  );

  useMount(async () => {
    try {
      const state = await getClipboardPreviewState();
      setPreviewState(state);
    } catch (error) {
      log.error("load preview state failed", error);
    }
  });

  useTauriListen<ClipboardPreviewState | null>(
    TAURI_EVENT.PREVIEW_UPDATED,
    (event) => {
      setPreviewState(event.payload);
    },
  );

  useTauriListen<ClipboardPreviewGesturePayload>(
    TAURI_EVENT.PREVIEW_GESTURE,
    (event) => {
      const gesture = event.payload;
      if (gesture.sessionId !== visibleSessionId) return;
      if (payload?.kind !== "image" && payload?.singleImagePreview !== true) {
        return;
      }

      setImageZoomState((current) => {
        const baseZoom =
          current.sessionId === gesture.sessionId ? current.zoom : 1;
        return {
          sessionId: gesture.sessionId,
          zoom: clampZoom(baseZoom * gesture.factor),
        };
      });
    },
  );

  useEffect(() => {
    if (!active || previewRequestId === null) return;

    void setClipboardPreviewPanelRect(previewRequestId, reportedPanelRect);
  }, [active, previewRequestId, reportedPanelRect]);

  useEffect(() => {
    setTextOverflowReadyKey(null);
    if (
      payload?.kind !== "text" ||
      visibleItemId === null ||
      textHeightState.itemId !== visibleItemId
    ) {
      return;
    }

    const timer = window.setTimeout(() => {
      setTextOverflowReadyKey(visibleSessionId);
    }, PREVIEW_TEXT_OVERFLOW_SETTLE_MS);

    return () => {
      window.clearTimeout(timer);
    };
  }, [payload?.kind, textHeightState.itemId, visibleItemId, visibleSessionId]);

  const handleBeforeDestroy = (event: { payload: BeforeDestroyPayload }) => {
    if (event.payload.label !== WINDOW_LABEL.PREVIEW) return;

    setPreviewState(null);
    setPayloadResetToken((current) => {
      return current + 1;
    });
  };

  const handleImageZoomChange = (zoom: number) => {
    setImageZoomState({ sessionId: visibleSessionId, zoom });
  };

  const handleTextContentHeightChange = useCallback(
    (itemId: string, height: number) => {
      setTextHeightState((current) => {
        if (current.itemId === itemId && current.height === height) {
          return current;
        }
        return { height, itemId };
      });
    },
    [],
  );

  useTauriListen<BeforeDestroyPayload>(
    TAURI_EVENT.WINDOW_BEFORE_DESTROY,
    handleBeforeDestroy,
  );

  if (!visibleState) {
    return <div className="fixed inset-0 overflow-hidden bg-transparent" />;
  }

  const isLoading = loadingItemId !== null;
  const payloadKey = payload ? cacheKey(payload, redactSecrets) : "empty";
  const { layout } = visibleState;
  const svgStyle = rectStyle(layout.overlayRect);

  return (
    <div className="fixed inset-0 overflow-hidden bg-transparent">
      <motion.svg
        animate={active ? "open" : "closed"}
        aria-hidden="true"
        className="pointer-events-none absolute inset-0 z-10 overflow-visible"
        initial="closed"
        role="presentation"
        style={svgStyle}
        transition={PREVIEW_PANEL_TRANSITION}
        variants={PREVIEW_CONNECTOR_VARIANTS}
        viewBox={`0 0 ${layout.overlayRect.width} ${layout.overlayRect.height}`}
      >
        <motion.path
          className="stroke-ant-primary"
          d={motionLayout.path}
          fill="none"
          strokeLinecap="round"
          strokeWidth="2"
        />
        <motion.circle
          className="fill-ant-container stroke-ant-primary"
          cx={motionLayout.sourceDotX}
          cy={motionLayout.sourceDotY}
          r={PREVIEW_CONNECTOR_DOT_RADIUS}
          strokeWidth="2.5"
        />
        <motion.circle
          className="fill-ant-container stroke-ant-primary"
          cx={motionLayout.targetDotX}
          cy={motionLayout.targetDotY}
          r={PREVIEW_CONNECTOR_DOT_RADIUS}
          strokeWidth="2.5"
        />
      </motion.svg>

      <div
        aria-hidden="true"
        className="pointer-events-none invisible absolute top-0 left-0 z-0 flex w-fit min-w-72 max-w-120 flex-col overflow-visible rounded-2 border border-ant-border bg-ant-container/95 shadow-lg backdrop-blur"
        ref={panelMeasureRef}
        style={panelMeasureStyle}
      >
        <PreviewHeader payload={payload} />

        {shouldRenderMeasuredContent(payload) && (
          <div className="min-h-0">
            <PreviewContent payload={payload} />
          </div>
        )}
      </div>

      <motion.div
        animate={active ? "open" : "closed"}
        className="absolute z-5 flex min-w-72 flex-col overflow-hidden rounded-2 border border-ant-border bg-ant-container/95 shadow-lg backdrop-blur"
        initial="closed"
        style={motionLayout.panelStyle}
        transition={PREVIEW_PANEL_TRANSITION}
        variants={PREVIEW_PANEL_VARIANTS}
      >
        <PreviewContentTransition contentKey={payloadKey}>
          <PreviewHeader payload={payload} />

          <div
            className={cn("min-h-0 flex-1 overflow-hidden transition-opacity", {
              "opacity-60": isLoading && payload !== null,
            })}
          >
            <PreviewContent
              imageZoom={imageZoom}
              onImageZoomChange={handleImageZoomChange}
              onTextContentHeightChange={handleTextContentHeightChange}
              payload={payload}
              textOverflowReady={
                payload?.kind === "text"
                  ? textOverflowReadyKey === visibleSessionId
                  : true
              }
            />
          </div>
        </PreviewContentTransition>

        {isLoading && (
          <div className="pointer-events-none absolute inset-0 flex items-center justify-center bg-ant-mask/10">
            <Spin size="small" />
          </div>
        )}
      </motion.div>
    </div>
  );
};

function shouldRenderMeasuredContent(
  payload: ReturnType<typeof usePreviewPayload>["payload"],
) {
  if (!payload) return true;

  return payload.kind === "image" || payload.singleImagePreview;
}

function clampZoom(value: number) {
  return Math.min(
    PREVIEW_IMAGE_ZOOM_MAX,
    Math.max(PREVIEW_IMAGE_ZOOM_MIN, value),
  );
}

export default Preview;
