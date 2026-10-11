import { DefaultXComponent } from '@ohos-rs/ability';

@Entry
@Component
struct Index {
  build() {
    Stack() {
      DefaultXComponent({ moduleName: 'hello_openharmony' })
    }
    .width('100%')
    .height('100%')
  }
}
