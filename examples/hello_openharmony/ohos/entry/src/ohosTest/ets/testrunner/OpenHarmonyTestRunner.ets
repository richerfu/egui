import { abilityDelegatorRegistry, TestRunner, Driver, PointerMatrix } from '@kit.TestKit';
import { hilog } from '@kit.PerformanceAnalysisKit';
import { BusinessError } from '@kit.BasicServicesKit';

export default class OpenHarmonyTestRunner implements TestRunner {
  onPrepare(): void {}
  async onRun(): Promise<void> {
    const delegator = abilityDelegatorRegistry.getAbilityDelegator();
    const args = abilityDelegatorRegistry.getArguments().parameters;
    const bundle = args['-s bundle'] || 'org.egui.hello';
    const x = Number(args['-s x'] || '1000');
    const y = Number(args['-s y'] || '1000');
    try {
      await delegator.startAbility({ bundleName: bundle, abilityName: 'EntryAbility' });
      const driver = Driver.create();
      await driver.delayMs(2000);
      const pressures = args['-s pressure'] ? [Number(args['-s pressure'])] : [0.25, 0.75, 1.0];
      for (const pressure of pressures) {
        hilog.info(0x0000, 'EguiPenTest', 'penSwipe pressure=%{public}s', pressure.toString());
        await driver.penSwipe({ x, y }, { x: x + 150, y: y + 100 }, 600, pressure);
      }
      if (!args['-s pressure']) {
        const points = PointerMatrix.create(1, 3);
        points.setPoint(0, 0, { x, y });
        points.setPoint(0, 1, { x: x + 60, y: y + 50 });
        points.setPoint(0, 2, { x: x + 120, y });
        await driver.injectPenPointerAction(points, 600, 0.5);
      }
      await delegator.finishTest('UITest pen injection completed; verify application event log', 0);
    } catch (error) {
      const err = error as BusinessError;
      await delegator.finishTest(`UITest pen injection failed ${err.code}: ${err.message}`, 1);
    }
  }
}
