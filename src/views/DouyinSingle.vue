<script lang="ts" setup>
import { reactive, ref} from 'vue'
import { dialog, invoke, shell } from '@tauri-apps/api'
import { appWindow } from '@tauri-apps/api/window'

import { LinkOutlined, SearchOutlined, EyeOutlined, DownloadOutlined } from '@ant-design/icons-vue'
import { ElMessage } from 'element-plus'
import { UserVideoInfo } from '../types/douyin'

const save_path = ref()  // 文件保存路径
const videoTable = ref(Array()) // 表格数据
// 搜索表单
const form = reactive({
  share_url: 'https://v.douyin.com/jpL1UwY/',
})
const percentage = ref(0) // 进度条百分比
const isDownloading = ref(false) // 是否下载中
const isSearching = ref(false)  // 是否搜索中
const isDownloadSuccess = ref(false); // 是否下载成功

// 下载
const onDownload = async (index: number) => {
    const unlisten = appWindow.listen('douyin_single_download', (data: any) => {
        percentage.value = data.payload.percentage
    })
    try{
      const save_dir = (await dialog.open({ directory: true}))
      if (!save_dir){
        unlisten.then((f)=> f())
        ElMessage.error("取消下载")
        return
      }
      isDownloading.value = true
      const info = videoTable.value[index]
      save_path.value = save_dir + "/" + info.video_title
      save_path.value = await invoke("douyin_single_download", { savePath: save_path.value, videoUrl: info.video_url})
      percentage.value = 0
      isDownloadSuccess.value = true
      ElMessage.success("下载成功")
    }catch (e) {
      ElMessage.error("下载失败, 错误:" + e)
    }finally{
      unlisten.then((f)=> f())
      isDownloading.value = false;
    }
}

// 搜索
const onSearch = async () => { 
  isSearching.value = true;
  try {
    videoTable.value.length = 0
    const info: UserVideoInfo = await invoke("douyin_single_search", { url: form.share_url })
    if (info.video_info.items.length > 0) {
      videoTable.value.length = 0
      videoTable.value.push({
        nickname: info.user_info.nickname,
        avatar_url: info.user_info.avatar_url,
        uid: info.user_info.uid,
        video_count: info.user_info.video_count,
        video_id: info.video_info.items[0].video_id,
        video_title: info.video_info.items[0].video_title,
        cover_url : info.video_info.items[0].cover_url,
        video_url: info.video_info.items[0].video_url
      })
    }else{
      ElMessage.info("未找到相关视频")
    }
  }catch (e) {
    ElMessage.error("错误:" + e)
  }finally {
    isDownloadSuccess.value = false
    isSearching.value = false
  }
}

// 打开已下载视频
const onOpen = async () => {
  await shell.open(save_path.value)
}

// 浏览器中预览视频
const onPreview = async (index: number) => {
  const data = videoTable.value[index]
  shell.open(data.video_url)
}

</script>




<template>
  <el-form 
    :inline="true" 
    :model="form" 
    class="video-search-form"
    >
    
    <el-form-item label="视频分享链接">
      <el-input
        v-model="form.share_url"
        class="video-search-input"
        autosize
        placeholder="https://v.douyin.com/23FsM5g/"
        :suffix-icon="LinkOutlined"
      />
    </el-form-item>
    <el-form-item label="">
      <el-button @click="onSearch" class="video-search-button" :icon="SearchOutlined" :disabled="isSearching || isDownloading">
        <el-row v-if="!isSearching">搜索</el-row>
        <el-row v-else>正在搜索</el-row>
      </el-button>
    </el-form-item>
  </el-form>

  <el-table v-show="videoTable.length" :data="videoTable">
    <el-table-column prop="video_title" label="标题" width="auto" />
    <el-table-column min-width="55"  prop="cover_url" label="封面">
      <template #default="scope">
      <el-image
      style="width: 60px; height: 60px"
      :src="scope.row.cover_url"
      :preview-src-list="[scope.row.cover_url]"
      preview-teleported="true"
      hide-on-click-modal="true"
      :initial-index="4"
      fit="cover"
    />
    </template>
    </el-table-column>
    <el-table-column prop="nickname" label="作者" width="auto" />
    <el-table-column  fixed="right" label="操作" width="auto">
      <template #default="scope">
        <el-button v-if="!isDownloadSuccess" link type="primary" size="small" @click="onDownload(scope.$index)" :icon="DownloadOutlined" :disabled="isDownloading">下载</el-button>
        <el-button v-else link type="primary" size="small" @click="onOpen()" :icon="DownloadOutlined" :disabled="!isDownloadSuccess">打开</el-button>
        <el-button link type="primary" size="small" @click="onPreview(scope.$index)" :icon="EyeOutlined">预览</el-button>
      </template>
    </el-table-column>
  </el-table>

  <div>
    <el-progress v-if="isDownloading" :text-inside="true" :stroke-width="20" :percentage="percentage" />
  </div>
  
</template>

<style scoped>
.video-search-form {
  width: 100%;
  text-align: center;
  margin: 0px;
}
.video-search-button {
  width: auto;
  border-radius: 20px
}
</style>
