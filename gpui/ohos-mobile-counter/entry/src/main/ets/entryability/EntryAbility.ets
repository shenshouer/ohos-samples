import { UIAbility } from '@kit.AbilityKit';
import { window } from '@kit.ArkUI';
import { hilog } from '@kit.PerformanceAnalysisKit';
import nativeProbe from 'libgpui_mobile_counter.so';

const DOMAIN = 0x0000;
const TAG = 'GPUI_M4_2';

export default class EntryAbility extends UIAbility {
  onWindowStageCreate(windowStage: window.WindowStage): void {
    windowStage.loadContent('pages/Index', (err) => {
      if (err.code) {
        hilog.error(DOMAIN, TAG, 'loadContent failed: %{public}d', err.code);
      } else {
        hilog.info(DOMAIN, TAG, 'loadContent succeeded');
      }
    });
  }

  onBackground(): void {
    nativeProbe.gpuiPause();
  }

  onForeground(): void {
    nativeProbe.gpuiResume();
  }
}
