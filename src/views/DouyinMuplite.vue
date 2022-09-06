<script lang="ts" setup>
import { reactive, ref} from 'vue'
import { dialog, invoke, shell } from '@tauri-apps/api'
import { LinkOutlined, SearchOutlined, EyeOutlined, DownloadOutlined, ClearOutlined } from '@ant-design/icons-vue'
import { ElMessage, ElTable } from 'element-plus'
import { appWindow } from '@tauri-apps/api/window'
import { round } from 'lodash'
import { UserVideoInfo, VideoInfo } from '../types/douyin'


const tableData = ref(Array())  // 表格数据
const tableRef = ref<InstanceType<typeof ElTable>>() // 表格引用
const selectedList = ref(Array())  // 选中列表
const isSearching = ref(false)  // 是否搜索中
const isDownloading = ref(false); // 是否下载中

// 搜索表单
const form = reactive({
  home_url: 'https://v.douyin.com/j3XPKMg/',
})

const total_count = ref(0)   // 下载文件总数
const finish_count = ref(0) //  已下载文件总数
const percentage = ref(0)  // 进度条百分比

const onSearch = async () => {

  // 监听搜索事件, 将数据插入表格末尾
  const unlisten = appWindow.listen('douyin_get_all_video_info', (data: any) => {
      let video_info: VideoInfo = data.payload
      for (let i = 0; i < video_info.items.length; i++) {
        tableData.value.push(video_info.items[i])
      }
      if (!video_info.has_more) {
        ElMessage.success(`搜索完成, 共找到${tableData.value.length}个视频.`)
      }
  })

  try {
    isSearching.value = true
    const data: UserVideoInfo = await invoke('douyin_muplit_search', { homeUrl: form.home_url})
    tableData.value = data.video_info.items
    await invoke('douyin_get_all_video_info', {
       uid: data.user_info.uid, 
       videoCount: data.user_info.video_count,
       maxCursor: data.video_info.max_cursor,
    })
  }catch (e) {
    ElMessage.error(e as string)
  }finally {
    unlisten.then((f)=> f())
    isSearching.value = false
  }
}

const onSelectionChange = (obj: any) => {
  selectedList.value = obj
}

const onClear = () => {
  tableRef.value!.clearSelection()
  selectedList.value = []
}

// 下载单个视频
const onDownloadItem = async (index: number) => {
    try{
      const save_dir = (await dialog.open({ directory: true}))
      if (!save_dir) {
        ElMessage.error("取消下载")
        return
      }
      tableData.value[index].is_downloading = true
      isDownloading.value = true
      const info = tableData.value[index]
      let save_path = save_dir + "/" + info.video_title + ".mp4"
      tableData.value[index].save_path = await invoke("douyin_single_download", { savePath: save_path, videoUrl: info.video_url})
      tableData.value[index].is_success = true
      ElMessage.success("下载成功")
    }catch (e) {
      ElMessage.error("下载失败, 错误:" + e)
      tableData.value[index].downloaded_success = false
    }finally{
      tableData.value[index].is_downloading = false;
    }
}

// 下载选中的视频列表
const onDownloadSelected = async () => {
  try {
    const save_dir = (await dialog.open({ directory: true}))
    if (!save_dir){
        ElMessage.error("取消下载")
        return
    }
    isDownloading.value = true
    await downlad(selectedList.value, save_dir as string)
  }catch (e) {
    ElMessage.error("下载失败, 错误:" + e)
  }finally {
    isDownloading.value = false
    finish_count.value = 0
    total_count.value = 0
    percentage.value = 0
  }
}

// 下载表格中已搜索出来的所有视频
const onDownloadAll = async () => { 
  try {
    const save_dir = (await dialog.open({ directory: true}))
    if (!save_dir){
        ElMessage.error("取消下载")
        return
    }
    isDownloading.value = true
    await downlad(tableData.value, save_dir as string)
  }catch (e) {
    ElMessage.error("下载失败, 错误:" + e)
  }finally {
    isDownloading.value = false
    finish_count.value = 0
    total_count.value = 0
    percentage.value = 0
  }
}

type DownloadNotifyData = {
  video_id: string,
  video_title: string,
  save_path: string,
  is_success: boolean,
}

const downlad = async (items: any, save_dir: string) => {
  const unlisten = appWindow.listen('douyin_muplit_download', (data: any) => {
      let result: DownloadNotifyData = data.payload
      for (let i = 0; i < tableData.value.length; i++) {
        if (tableData.value[i].video_id === result.video_id) {
            tableData.value[i].is_success = result.is_success;
            tableData.value[i].save_path = result.save_path;
        }
      }
      finish_count.value += 1;
      percentage.value = round(finish_count.value / total_count.value * 100, 2);
  })
  try {
    const video_id_list = items.map( (e: { video_id: string }) => e.video_id)
    total_count.value = video_id_list.length
    finish_count.value = 0
    for (let i = 0; i < tableData.value.length; i++) {
        if (video_id_list.includes(tableData.value[i].video_id)) {
          tableData.value[i].is_downloading = true
        }
    }
    await invoke("douyin_muplit_download", { items: items, saveDir: save_dir})
    isDownloading.value = false
    ElMessage.success("下载完成")
  }catch (e) {
    ElMessage.error("下载失败, 错误:" + e)
  }
}

// 在浏览器中预览
const onPreview = async (index: number) => {
  const data = tableData.value[index]
  shell.open(data.video_url)
}

// 打开已下载的视频
const onOpen = async (index: number) => {
  const data = tableData.value[index]
  shell.open(data.save_path)
}

</script>

<template>

    <el-form 
    :inline="true" 
    :model="form" 
    class="video-search-form"
    >
    <el-form-item label="用户主页链接">
      <el-input
        v-model="form.home_url"
        class="video-search-input"
        autosize
        placeholder="https://v.douyin.com/23FsM5g/"
        :suffix-icon="LinkOutlined"
      />
    </el-form-item>
    <el-form-item label="">
      <el-button @click="onSearch" class="video-search-button" :icon="SearchOutlined" :disabled="isSearching  || isDownloading">
        <el-row v-if="!isSearching">搜索</el-row>
        <el-row v-else>正在搜索</el-row>
      </el-button>
    </el-form-item>
  </el-form>
  
  

<el-col v-show="tableData.length">

  <el-row>
    <el-select
      fit-input-width="true"
      class="operate-select"
      max-height="100"
      :placeholder="`共选中${selectedList.length}条记录`"
      disabled="true"
    >
    </el-select>
    <el-button @click="onClear" :icon="ClearOutlined" class="operate-button" :disabled="selectedList.length==0">清空选中</el-button>
    <el-button @click="onDownloadSelected" :icon="DownloadOutlined" class="operate-button" :disabled="selectedList.length==0 || isDownloading">下载选中</el-button>
    <el-button @click="onDownloadAll" :icon="DownloadOutlined" class="operate-button" :disabled="isDownloading">下载全部</el-button>
  </el-row>

  <el-row>
  <el-table
    ref="tableRef"
    :data="tableData"
    border
    max-height="500px"
    class="tableStyle"
    scrollbar-always-on
    reserve-selection="true"
    @selection-change="onSelectionChange"
  >
    <el-table-column type="selection" width="55" />
    
    <el-table-column min-width="55"  prop="cover_url" label="封面" align="center">
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

    <el-table-column prop="video_title" label="标题" width="auto" align="center"/>

    <el-table-column  fixed="right" label="操作" width="auto" align="center">
      <template #default="scope">
        <el-button v-if="!scope.row.is_success" link type="primary" size="small" @click="onDownloadItem(scope.$index)" :icon="DownloadOutlined" :disabled="scope.row.is_downloading">
          <el-row v-if="!scope.row.is_downloading">下载</el-row>
          <el-row v-else>下载中</el-row>
        </el-button>
        <el-button v-else link type="primary" size="small" @click="onOpen(scope.$index)" :icon="DownloadOutlined">打开</el-button>
        <el-button link type="primary" size="small" @click="onPreview(scope.$index)" :icon="EyeOutlined">预览</el-button>
      </template>
    </el-table-column>
  </el-table>
  </el-row>
</el-col>

<div>
  <el-row style="margin-top: 10px;" v-if="isDownloading">
    <el-col :span="22"><el-progress :text-inside="true" :stroke-width="20" :percentage="percentage" /></el-col>
    <el-col :span="2"><label>{{ finish_count }}/{{ total_count }}</label></el-col>
  </el-row>
  
  
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

.operate-button {
  width: auto;
  border-radius: 20px
}

.operate-select {
  width: 150px;
  margin-right: 10px;
  border: 1px;
  border-radius: 20px;
}
.el-progress{width:100%;}
</style>