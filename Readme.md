## 关于

参照[Let&#39;s build a browser engine! Part 1: Getting started](https://limpet.net/mbrubeck/2014/08/08/toy-layout-engine-1.html)写的玩具浏览器项目；

## 注意的地方

### boa

- 如果发现 `boa_engine = "0.16.0"`死活装不上，估计是rust版本过低，可以先升级版本：

  ```sh
  rustup update
  ```

### ggez

- ggez 0.8.1版本在某些rust版本下编译后会出现canvas绘制后一片空白，可以通过升级ggez版本来解决[^1]；



[^1]:  https://stackoverflow.com/questions/76404202/ggez-rectangle-doesnt-show-up-on-canvas?newreg=c85fe6264ece4c708800bc3250bfdc94
