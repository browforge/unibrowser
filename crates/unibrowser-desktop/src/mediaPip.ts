import { t } from "./i18n";
import type { UiCommandDto } from "./commandManager";

const PIP_COMMAND = "unibrowser.media.pictureInPicture";

export function patchPictureInPictureCommand(cmd: UiCommandDto): UiCommandDto {
  if (cmd.command_id !== PIP_COMMAND || !cmd.enabled) {
    return cmd;
  }
  const doc = document as Document & { pictureInPictureEnabled?: boolean };
  if (!doc.pictureInPictureEnabled) {
    return {
      ...cmd,
      enabled: false,
      reason: t("pip.unsupportedWebview"),
    };
  }
  return cmd;
}

export async function tryMediaPictureInPicture(): Promise<void> {
  const media = document.querySelector<HTMLVideoElement | HTMLAudioElement>(".media-view");
  if (!media) {
    throw new Error(t("pip.noMedia"));
  }
  const doc = document as Document & { pictureInPictureEnabled?: boolean };
  if (!doc.pictureInPictureEnabled) {
    throw new Error(t("pip.unsupportedWebview"));
  }
  if (document.pictureInPictureElement) {
    await document.exitPictureInPicture();
    return;
  }
  if (media instanceof HTMLVideoElement) {
    await media.requestPictureInPicture();
    return;
  }
  const audioPip = media as HTMLAudioElement & {
    requestPictureInPicture?: () => Promise<PictureInPictureWindow>;
  };
  if (typeof audioPip.requestPictureInPicture === "function") {
    await audioPip.requestPictureInPicture();
    return;
  }
  throw new Error(t("pip.audioUnsupported"));
}
