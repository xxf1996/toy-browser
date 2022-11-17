基于[fontdue](https://github.com/mooman219/fontdue)可以计算文字排版，以便得出指定文字占用的矩形区域信息；据此可以计算inline元素的布局信息，因为inline元素的高度完全由文字内容进行计算[^1]；



目前有一个问题fontdue计算文字布局倒是没问题，但是目前好像没找到将完整字符串整个光栅化的API[^2]？目前只看到单个字符的光栅化……



## 字体来源

- [思源黑体 (行高修正版)-字体免费下载-文悦字库官网](https://wenyue.cn/fonts/1504)
- [Chinese Fonts: An Open Source, Personal and Non-commercial Collection - LingData | Linguistic Datasets](https://lingdata.org/archives/fonts.html)
- [atelier-anchor/smiley-sans: 得意黑 Smiley Sans：一款在人文观感和几何特征中寻找平衡的中文黑体](https://github.com/atelier-anchor/smiley-sans)



[^1]: https://www.w3.org/TR/CSS2/visudet.html#inline-non-replaced
[^2]: [Rasterize a whole string · Issue #128 · mooman219/fontdue](https://github.com/mooman219/fontdue/issues/128)
