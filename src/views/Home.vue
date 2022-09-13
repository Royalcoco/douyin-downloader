<script setup lang="ts">
import { GithubFilled } from '@ant-design/icons-vue';
import { shell } from '@tauri-apps/api'

const onClick = () => {
    shell.open('https://github.com/ClassmateLin/video-downloader')
}
</script>

<template>
    <div class="content">
        <el-col :span="12">
            <el-row>
               <el-button @click="onClick"><el-icon class="ali-icon-shipinxiazai"></el-icon>Video downloader</el-button>
            </el-row>
            <el-row>
                <p>基于rust+tauri开发的一款抖音视频下载器, 支持批量下载无水印视频。</p>
            </el-row>
        </el-col>
    </div>
</template>

<style scoped>
.content {
    height: 200px;
    display: flex;
    align-items: center;
    justify-content: center;
}


</style>
