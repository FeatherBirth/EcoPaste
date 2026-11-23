import { Empty } from "antd";
import type { TFunction } from "i18next";
import { type FC, useLayoutEffect, useRef, type WheelEvent } from "react";
import { useTranslation } from "react-i18next";
import { Virtuoso } from "react-virtuoso";
import type {
  ClipboardPreviewFileEntry,
  ClipboardPreviewPayload,
} from "@/commands";
import AssetImage from "@/components/AssetImage";
import VirtuosoScroller, {
  type VirtuosoScrollerChildrenProps,
} from "@/components/VirtuosoScroller";
import { cn } from "@/utils/cn";
import {
  PREVIEW_IMAGE_ZOOM_MAX,
  PREVIEW_IMAGE_ZOOM_MIN,
  PREVIEW_IMAGE_ZOOM_STEP,
} from "../constants";

export interface PreviewContentProps {
  imageZoom?: number;
  onImageZoomChange?: (zoom: number) => void;
  onTextContentHeightChange?: (itemId: string, height: number) => void;
  payload: ClipboardPreviewPayload | null;
}

export interface PreviewHeaderProps {
  payload: ClipboardPreviewPayload | null;
}

interface PayloadViewerProps {
  payload: ClipboardPreviewPayload;
}

interface ImageViewerProps extends PayloadViewerProps {
  onZoomChange: (zoom: number) => void;
  zoom: number;
}

interface FilesViewerProps extends ImageViewerProps {}

interface TextViewerProps extends PayloadViewerProps {
  onContentHeightChange?: (itemId: string, height: number) => void;
}

interface FilePreviewRowProps {
  file: ClipboardPreviewFileEntry;
}

const FILES_VIRTUOSO_COMPONENTS = {
  Footer: PreviewFilesPadding,
  Header: PreviewFilesPadding,
};

/**
 * Content Viewer 顶部元信息区。
 */
export const PreviewHeader: FC<PreviewHeaderProps> = (props) => {
  const { payload } = props;
  const { t } = useTranslation(["preview", "clipboard"]);
  const title = payload ? previewTitle(t, payload) : t("title.loading");
  const meta = payload ? previewMeta(t, payload) : t("meta.contentViewer");
  const typeKey = payload ? (payload.subKind ?? payload.kind) : null;
  const typeLabel = typeKey ? t(`clipboard:types.${typeKey}`) : "";

  return (
    <div className="flex h-12 shrink-0 items-center justify-between gap-3 border-ant-border border-b px-4">
      <div className="min-w-0">
        <div className="truncate font-medium text-sm">{title}</div>
        <div className="truncate text-ant-secondary text-xs">{meta}</div>
      </div>

      {payload && (
        <span className="shrink-0 rounded-1 bg-ant-fill-secondary px-2 py-0.5 text-ant-secondary text-xs">
          {typeLabel}
        </span>
      )}
    </div>
  );
};

/**
 * 按 payload kind 分发到基础 viewer。
 */
export const PreviewContent: FC<PreviewContentProps> = (props) => {
  const {
    imageZoom = 1,
    onImageZoomChange = noopZoomChange,
    onTextContentHeightChange,
    payload,
  } = props;
  const { t } = useTranslation("preview");

  if (!payload) {
    return (
      <div className="flex min-h-24 items-center justify-center">
        <Empty
          description={t("empty.content")}
          image={Empty.PRESENTED_IMAGE_SIMPLE}
        />
      </div>
    );
  }

  if (payload.kind === "image") {
    return (
      <ImageViewer
        key={payload.id}
        onZoomChange={onImageZoomChange}
        payload={payload}
        zoom={imageZoom}
      />
    );
  }

  if (payload.kind === "files") {
    return (
      <FilesViewer
        onZoomChange={onImageZoomChange}
        payload={payload}
        zoom={imageZoom}
      />
    );
  }

  return (
    <TextViewer
      onContentHeightChange={onTextContentHeightChange}
      payload={payload}
    />
  );
};

/**
 * 文本预览：所有文本族内容使用单个只读 textarea，支持原生选择与精确测量溢出。
 */
const TextViewer: FC<TextViewerProps> = (props) => {
  const { onContentHeightChange, payload } = props;
  const { t } = useTranslation("preview");
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const text = payload.text ?? "";

  // biome-ignore lint/correctness/useExhaustiveDependencies: remeasure after payload text changes
  useLayoutEffect(() => {
    const textarea = textareaRef.current;
    if (!textarea || !onContentHeightChange) return;

    const reportHeight = () => {
      onContentHeightChange(payload.id, textarea.scrollHeight);
    };
    reportHeight();

    const observer = new ResizeObserver(reportHeight);
    observer.observe(textarea);

    return () => {
      observer.disconnect();
    };
  }, [onContentHeightChange, payload.id, text]);

  if (text.length === 0) {
    return (
      <div className="flex min-h-24 items-center justify-center">
        <Empty
          description={t("empty.text")}
          image={Empty.PRESENTED_IMAGE_SIMPLE}
        />
      </div>
    );
  }

  return (
    <textarea
      className="block h-full min-h-0 w-full flex-1 select-text resize-none overflow-auto whitespace-pre-wrap border-0 bg-transparent p-4 font-mono text-xs leading-5.5 outline-none"
      readOnly
      ref={textareaRef}
      spellCheck={false}
      value={text}
      wrap="soft"
    />
  );
};

/**
 * 图片预览：使用原图路径渲染，缺失时降级为空状态。
 */
const ImageViewer: FC<ImageViewerProps> = (props) => {
  const { onZoomChange, payload, zoom } = props;
  const { t } = useTranslation("preview");
  const imageWidth = payload.imageWidth ?? void 0;
  const imageHeight = payload.imageHeight ?? void 0;

  if (!payload.imagePath || !payload.imageExists) {
    return (
      <div className="flex min-h-24 items-center justify-center">
        <Empty
          description={t("empty.imageMissing")}
          image={Empty.PRESENTED_IMAGE_SIMPLE}
        />
      </div>
    );
  }

  const handleWheel = (event: WheelEvent<HTMLDivElement>) => {
    event.preventDefault();

    const factor = event.ctrlKey
      ? clamp(
          Math.exp(-event.deltaY * 0.01),
          1 / PREVIEW_IMAGE_ZOOM_STEP,
          PREVIEW_IMAGE_ZOOM_STEP,
        )
      : event.deltaY < 0
        ? PREVIEW_IMAGE_ZOOM_STEP
        : 1 / PREVIEW_IMAGE_ZOOM_STEP;
    onZoomChange(
      clamp(zoom * factor, PREVIEW_IMAGE_ZOOM_MIN, PREVIEW_IMAGE_ZOOM_MAX),
    );
  };

  return (
    <div
      className="flex min-h-0 w-full flex-1 items-center justify-center overflow-hidden p-4"
      onWheel={handleWheel}
    >
      <AssetImage
        alt={t("image.alt")}
        className="block size-full object-contain"
        draggable={false}
        height={imageHeight}
        src={payload.imagePath}
        width={imageWidth}
      />
    </div>
  );
};

function noopZoomChange(_zoom: number) {}

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}

/**
 * 文件预览：虚拟列表展示路径、文件名、存在状态与基础大小。
 */
const FilesViewer: FC<FilesViewerProps> = (props) => {
  const { onZoomChange, payload, zoom } = props;
  const { t } = useTranslation("preview");

  if (payload.singleImagePreview && payload.imagePath && payload.imageExists) {
    return (
      <ImageViewer
        key={payload.id}
        onZoomChange={onZoomChange}
        payload={payload}
        zoom={zoom}
      />
    );
  }

  if (payload.files.length === 0) {
    return (
      <div className="flex min-h-24 items-center justify-center">
        <Empty
          description={t("empty.files")}
          image={Empty.PRESENTED_IMAGE_SIMPLE}
        />
      </div>
    );
  }

  return <VirtuosoScroller>{renderFilesVirtuoso}</VirtuosoScroller>;

  function renderFilesVirtuoso(props: VirtuosoScrollerChildrenProps) {
    const { scrollerRef } = props;
    const components =
      payload.totalFiles > payload.files.length
        ? {
            Footer: renderFilesFooter,
            Header: PreviewFilesPadding,
          }
        : FILES_VIRTUOSO_COMPONENTS;

    return (
      <Virtuoso
        components={components}
        computeItemKey={computeFileRowKey}
        itemContent={renderFileRow}
        scrollerRef={scrollerRef}
        totalCount={payload.files.length}
      />
    );
  }

  function computeFileRowKey(index: number) {
    return payload.files[index]?.path ?? index;
  }

  function renderFileRow(index: number) {
    const file = payload.files[index];
    if (!file) return <div className="h-10" />;

    return (
      <div className="px-2">
        <FilePreviewRow file={file} />
      </div>
    );
  }

  function renderFilesFooter() {
    return (
      <div className="px-4 py-2 text-ant-secondary text-xs">
        {t("file.shownCount", {
          shown: payload.files.length,
          total: payload.totalFiles,
        })}
      </div>
    );
  }
};

/**
 * 虚拟文件列表顶部留白。
 */
function PreviewFilesPadding() {
  return <div className="h-2" />;
}

/**
 * 文件 viewer 的单行展示。
 */
const FilePreviewRow: FC<FilePreviewRowProps> = (props) => {
  const { file } = props;
  const { t } = useTranslation("preview");
  const kindLabel = file.isDir ? t("file.folder") : t("file.item");
  const sizeLabel = file.size === null ? kindLabel : formatBytes(file.size);

  return (
    <div
      className={cn(
        "flex min-h-10 items-center gap-2 rounded-1.5 px-2 py-1.5",
        { "opacity-50": !file.exists },
      )}
      title={file.path}
    >
      {file.iconPath ? (
        <AssetImage className="size-6 shrink-0" src={file.iconPath} />
      ) : (
        <i
          aria-hidden
          className="i-lucide:file size-5 shrink-0 text-ant-secondary"
        />
      )}

      <div className="min-w-0 flex-1">
        <div
          className={cn("truncate text-xs", {
            "line-through": !file.exists,
          })}
        >
          {file.name}
        </div>
        <div className="truncate text-ant-secondary text-xs">
          {file.exists ? file.path : t("file.missingPath")}
        </div>
      </div>

      <span className="shrink-0 text-ant-secondary text-xs">{sizeLabel}</span>
    </div>
  );
};

/**
 * 生成 Content Viewer 标题。
 */
function previewTitle(
  t: TFunction<"preview">,
  payload: ClipboardPreviewPayload,
) {
  if (payload.kind === "files") {
    return t("title.files", { count: payload.totalFiles });
  }

  if (payload.kind === "image") {
    return t("title.image");
  }

  return t("title.text");
}

/**
 * 生成 Content Viewer 元信息。
 */
function previewMeta(
  t: TFunction<"preview">,
  payload: ClipboardPreviewPayload,
) {
  if (payload.kind === "files") {
    return t("meta.filesLoaded", { count: payload.files.length });
  }

  if (payload.kind === "image") {
    const dimensions =
      payload.imageWidth && payload.imageHeight
        ? `${payload.imageWidth} x ${payload.imageHeight}`
        : t("meta.unknownSize");
    const size = payload.size === null ? "" : ` · ${formatBytes(payload.size)}`;

    return `${dimensions}${size}`;
  }

  return t("meta.characters", {
    count: payload.size ?? payload.text?.length ?? 0,
  });
}

/**
 * 格式化字节大小为紧凑文本。
 */
function formatBytes(value: number) {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let size = value;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex += 1;
  }

  const fractionDigits = unitIndex === 0 || size >= 10 ? 0 : 1;

  return `${size.toFixed(fractionDigits)} ${units[unitIndex]}`;
}
