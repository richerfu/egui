import { MenuPlugin } from '@ohos-rs/ability-plugin-menu';
import { NotificationPlugin } from '@ohos-rs/ability-plugin-notification';
import { ProcessPlugin } from '@ohos-rs/ability-plugin-process';
import { SystemStatePlugin } from '@ohos-rs/ability-plugin-system-state';
import { AccessibilityPlugin } from '@ohos-rs/ability-plugin-accessibility';
import { PermissionPlugin } from '@ohos-rs/ability-plugin-permission';
import { AppControlPlugin } from '@ohos-rs/ability-plugin-app-control';
import { FilesPlugin } from '@ohos-rs/ability-plugin-files';
import { UrlPlugin } from '@ohos-rs/ability-plugin-url';
import { ClipboardPlugin } from '@ohos-rs/ability-plugin-clipboard';
import { WindowPlugin } from '@ohos-rs/ability-plugin-window';
import { KeyboardAvoidMode } from '@kit.ArkUI';
import { BridgeLifecycleEvent, BridgePluginHookContext, LazyPlugin, NativeAbility } from '@ohos-rs/ability';

class EguiWindowPlugin extends WindowPlugin {
  override async onLifecycle(event: BridgeLifecycleEvent, context: BridgePluginHookContext): Promise<void> {
    await super.onLifecycle(event, context);
    if (event.kind === 'ui-context-ready') {
      // Resize the native canvas so egui can lay out the visible editor. ArkUI's
      // default OFFSET mode can move a whole XComponent above the screen.
      context.getUIContext().setKeyboardAvoidMode(KeyboardAvoidMode.RESIZE);
    }
  }
}

export default class EntryAbility extends NativeAbility {
  public moduleName: string = 'hello_openharmony';
  public defaultPage: boolean = true;
  public bridgePlugins = [
    new LazyPlugin(() => new MenuPlugin()),
    new LazyPlugin(() => new NotificationPlugin({ scheme: 'egui-test' })),
    new LazyPlugin(() => new ProcessPlugin()),
    new LazyPlugin(() => new SystemStatePlugin()),
    new LazyPlugin(() => new AccessibilityPlugin()),

    new LazyPlugin(() => new PermissionPlugin()),
    new LazyPlugin(() => new EguiWindowPlugin()),
    new LazyPlugin(() => new ClipboardPlugin()),
    new LazyPlugin(() => new UrlPlugin()),
    new LazyPlugin(() => new FilesPlugin()),
    new LazyPlugin(() => new AppControlPlugin()),
  ];

}
