import { hilog } from '@kit.PerformanceAnalysisKit';
import { common } from '@kit.AbilityKit';
import { window } from '@kit.ArkUI';
import nativeProbe from 'libgpui_mobile_counter.so';

const DOMAIN = 0x0000;
const TAG = 'GPUI_M3_1';

@Entry
@Component
struct Index {
  @State probeResult: string = 'waiting';
  @State showSurface: boolean = true;
  @State narrowSurface: boolean = false;
  @State gpuiStatus: string = 'starting';
  private pumpTimer: number = -1;

  aboutToAppear(): void {
    const bits: number = nativeProbe.probe();
    this.probeResult = `probe bits: ${bits}`;
    hilog.info(DOMAIN, TAG, 'Rust probe bits: %{public}d', bits);
    const started: number = nativeProbe.gpuiStart();
    this.gpuiStatus = `GPUI embedded: ${started === 1 ? 'started' : 'failed'}`;
    this.pumpTimer = setInterval(() => { nativeProbe.gpuiPump(); }, 50);
  }

  aboutToDisappear(): void {
    if (this.pumpTimer !== -1) {
      clearInterval(this.pumpTimer);
      this.pumpTimer = -1;
    }
  }

  private async changeOrientation(landscape: boolean): Promise<void> {
    const context = this.getUIContext().getHostContext() as common.UIAbilityContext;
    try {
      const mainWindow = await window.getLastWindow(context);
      await mainWindow.setPreferredOrientation(landscape ? window.Orientation.LANDSCAPE : window.Orientation.PORTRAIT);
      hilog.info(DOMAIN, TAG, 'orientation set to %{public}s', landscape ? 'landscape' : 'portrait');
    } catch (error) {
      hilog.error(DOMAIN, TAG, 'orientation change failed: %{public}s', String(error));
    }
  }

  build() {
    Column() {
      Text('GPUI mobile host: M3-1')
        .fontSize(20)
      Text(this.probeResult)
        .fontSize(16)
      Text(this.gpuiStatus)
        .fontSize(16)
      Button(this.showSurface ? 'Hide Surface' : 'Show Surface')
        .onClick(() => { this.showSurface = !this.showSurface; })
      Button('Resize Surface')
        .onClick(() => { this.narrowSurface = !this.narrowSurface; })
      Row() {
        Button('Landscape')
          .onClick(() => { this.changeOrientation(true); })
        Button('Portrait')
          .onClick(() => { this.changeOrientation(false); })
      }
      if (this.showSurface) {
        XComponent({ id: 'gpui_surface', type: 'surface', libraryname: 'gpui_mobile_counter' })
          .width(this.narrowSurface ? '80%' : '100%')
          .layoutWeight(1)
      }
    }
    .width('100%')
    .height('100%')
  }
}
