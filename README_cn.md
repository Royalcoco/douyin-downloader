
<h1 align="center">
  <img src="src-tauri/icons/128x128.png" width="128" />
  <br>
  Video Downloader
  <br>
</h1>

<h3 align="center">
 一款跨平台的视频(抖音/B站)下载器, 使用<a href="https://github.com/tauri-apps/tauri">tauri</a>构建.
</h3>

<div align="center">

[![Windows Support](https://img.shields.io/badge/Windows-0078D6?style=flat&logo=windows&logoColor=white)](https://github.com/lzdyes/douyin-downloader/releases)
[![MacOS Support](https://img.shields.io/badge/MACOS-adb8c5?style=flat&logo=macos&logoColor=white)](https://github.com/lzdyes/douyin-downloader/releases)
[![Linux Support](https://img.shields.io/badge/linux-1793D1?style=flat&logo=linux&logoColor=white)](https://github.com/lzdyes/douyin-downloader/releases)

</div>

## 安装

- [Release](https://github.com/ClassmateLin/video-downloader/releases)页面下载对应的安装包，当然也可以自行编译，支持Windows x64, Linux x86_64 and macOS 11+。



## 开发

**请确保你已经安装好了Rust和Nodejs.**

- 拉取代码并安装依赖:

```
$ git clone https://github.com/ClassmateLin/video-downloader.git && cd video-downloader
$ npm install
```

- 启动开发环境服务: `npm run tauri dev`


- 编译安装包: `npm run tauri build`


## 🎉 功能 / 待做


### 抖音

#### 单个视频下载

- [x] 通过分享链接搜索视频. 例如: `https://v.douyin.com/jpL1UwY/`
- [x] 预览搜索出来的视频。
- [x] 下载搜索出来的视频。

#### 多个视频下载

- [x] 通过用户主页链接搜索所有视频. eg: `https://v.douyin.com/j3XPKMg/`
- [x] 批量下载全部视频。
- [x] 批量下载选中视频。
- [x] 下载/预览表格中某个视频。


### Bilibili

- [ ] 普通内容下载(Web|TV|App)


## 截图


![index](./docs/imgs/index.png)

![douyin_single_search](./docs/imgs/douyin_single_search.png)

![douyin_single_download](./docs/imgs/douyin_single_download.png)

![douyin_multi_search](./docs/imgs/douyin_muplit_search.png)

![douyin_single_download](./docs/imgs/douyin_muplit_download.png)


## 其他


- 通过分享链接下载视频:

 手机版抖音在视频页面, 点击右下角分享按钮->复制链接, 得到内容:
```
8.23 uSy:/ 复制打开抖音，看看【高冷观剧的作品】# 电视剧推荐 # dou在家追剧 https://v.douyin.com/6Y9eU3S/
```
复制`https://v.douyin.com/6Y9eU3S/`输入到软件中点击搜索。

- 通过用户主页下载视频:

手机版抖音点击用户头像即可进入用户主页, 点击右上角的三个点, 点击分享主页, 点击复制链接, 得到内容:

```
8- 长按复制此条消息，打开抖音搜索，查看TA的更多作品。 https://v.douyin.com/6Y9mvx7/
```
复制`https://v.douyin.com/6Y9mvx7/`输入到软件中，点击搜索按钮, 第一页的搜索是同步的, 视频列表有内容之后, 后台还在继续搜索剩余视频, 直到弹框提示搜索完成/搜索按钮变成可点击状态, 才搜索完所有视频。