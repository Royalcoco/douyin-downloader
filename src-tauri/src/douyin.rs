use std::{time::Duration, path::Path, sync::Arc};
use futures::future::join_all;
use serde_json::Value;
use tauri::{regex::Regex, Window};
use serde::{Serialize, Deserialize};
use tokio::{time::sleep, sync::mpsc};
use crate::downloader::Downloader;
use thiserror::Error;
use anyhow::Result;
use crate::common::USER_AGNET;

// 用户信息
#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub nickname: String,
    pub uid: String,
    pub avatar_url: String,
    pub video_count: u16,
}

// 单个视频信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoInfoItem {
    pub video_id: String,   // 视频ID
    pub video_title: String, // 视频标题
    pub video_url: String,  // 视频链接
    pub cover_url: String, // 视频封面URL
}


// 视频信息列表
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoInfo {
    pub max_cursor: u64,
    pub has_more: bool,  
    pub items: Vec<VideoInfoItem>,
}


// 用户和视频信息
#[derive(Debug, Serialize, Deserialize)]
pub struct UserVideoInfo {
    user_info: UserInfo,
    video_info: VideoInfo,
}


// 错误定义
#[derive(Error, Debug)]
enum DyError {
    
    #[error("系统错误")]
    SystemAbnormal,

    #[error("未找到视频")]
    VideoInfoNotFound,
    
    #[error("请求失败")]
    RequestFailed,

    #[error("解析数据失败")]
    ParseDataFailed,

    #[error("获取视频信息失败")]
    GetVideoInfoFailed,

    #[error("获取用户信息失败")]
    GetUserInfoFailed,

    #[error("下载视频失败")]
    DownloadVideoFailed,

    #[error("未找到用户")]
    UserInfoNotFound,
}

// 单文件下载进度条数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DySingleDownloadProgress {
    pub percentage: u8
}

// 多个文件下载进度条数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DyMuplitDownloadProgress {
    pub video_id: String,
    pub video_title: String,
    pub save_path: String,
    pub is_success: bool,
}

// 根据视频分享链接/用户主页URL解析ID
fn get_id_from_url(url: &mut String) -> String {

    if !url.contains('?') {
        url.push('?');

    }
    let regex = Regex::new(r"/[video|user]+/(?P<aweme_id>\S+)[/|\?]+").unwrap();
    
    match regex.captures(url) {
        Some(cap) => {
            cap.name("aweme_id").unwrap().as_str().replace('/', "")
        },
        None => "".to_string()
    }
}


/// 解析分享链接,获取用户信息和视频信息
#[tauri::command]
pub async fn douyin_single_search(url: String) -> Result<UserVideoInfo, String> {

    let client = reqwest::Client::builder()
        .user_agent(USER_AGNET)
        .build()
        .map_err(|_|{DyError::SystemAbnormal.to_string()})?;

    let mut real_url = client.get(&url)
        .send()
        .await
        .map_err(|_| DyError::RequestFailed.to_string())?
        .url()
        .to_string();

    let aweme_id = get_id_from_url(&mut real_url);
    if aweme_id.is_empty() {
        return Err(DyError::VideoInfoNotFound.to_string());
    }
    let api_url = format!("https://www.iesdouyin.com/web/api/v2/aweme/iteminfo/?item_ids={}", aweme_id);
    
    let data:Value = client.get(&api_url)
        .send()
        .await
        .map_err(|_| DyError::RequestFailed.to_string())?
        .json::<Value>()
        .await
        .map_err(|_| DyError::ParseDataFailed.to_string())?
        ["item_list"][0].clone();

    let video_id = data["aweme_id"]
        .to_string()
        .replace('"', "");

    let mut video_title = data["share_info"]["share_title"]
        .to_string()
        .replace('"', "")
        .split('@')
        .collect::<Vec<&str>>()[0]
        .to_string();

    if video_title.is_empty() {
        video_title = format!("无标题: {}", video_id);
    }

    let video_url = data["video"]["play_addr"]["url_list"][0]
        .to_string()
        .replace('"', "")
        .replace("playwm", "play")
        .replace("ratio=720p", "ratio=1080p").replace('"', "");
    
    let video_cover_url:String = data["video"]["origin_cover"]["url_list"][0]
        .to_string()
        .replace('"', "");

    let author_uid = data["author"]["uid"]
        .to_string()
        .replace('"', "");

    let author_name = data["author"]["nickname"]
        .to_string()
        .replace('"', "");

    let author_avatar = data["author"]["avatar_thumb"]["url_list"][0]
        .to_string()
        .replace('"', "");

    let user_info =  UserInfo { 
        nickname: author_name, 
        uid: author_uid, 
        avatar_url: author_avatar,
        video_count: 1
    };

    let video_list = vec![
        VideoInfoItem {
            video_id,
            video_title,
            video_url,
            cover_url: video_cover_url,
        }
    ];
    let video_info = VideoInfo { 
        max_cursor: 0,
        has_more: false, 
        items: video_list
    };

    let res = UserVideoInfo { user_info, video_info };

    Ok(res)

}


// 单个文件下载
#[tauri::command]
pub async fn douyin_single_download(save_path: String, video_url: String, window: Window) -> Result<String, String> {
    
    let downloader = Downloader::new(video_url, save_path, Some(8))
        .await
        .map_err(|_|DyError::SystemAbnormal.to_string())?;

    let save_path = downloader.get_save_path();
    let downloader_clone = downloader.clone();
    
    // 进度条
    tokio::spawn(async move {
        let total_size = downloader_clone.total_size();
        loop {
            let cur_size  = downloader_clone.downloaded_size().await;
            if cur_size >= total_size {
                let _ = window.emit("douyin_single_download", DySingleDownloadProgress{ percentage: 100 });
                break;
            }
            let percentage = (cur_size as f64 * 100.0 / total_size as f64 ).round() as u8;
            let _ = window.emit("douyin_single_download", DySingleDownloadProgress{ percentage });
            sleep(Duration::from_millis(100)).await;
        }
    });

    // 下载
    match downloader.download().await {
        Ok(_) => {
            sleep(Duration::from_millis(100)).await
        },
        Err(_) => return Err(DyError::DownloadVideoFailed.to_string()),
    };
    Ok(save_path)
}


// 获取用户信息
async fn get_user_info(uid: &String) -> Result<UserInfo> {
    
    let api_url = format!("https://www.iesdouyin.com/web/api/v2/user/info/?sec_uid={}", uid);

    let data = reqwest::Client::builder()
        .user_agent(USER_AGNET)
        .build()?
        .get(&api_url)
        .send()
        .await?
        .json::<Value>()
        .await?;

    
    let nickname = data["user_info"]["nickname"]
        .to_string()
        .replace('"', "");
        
    let video_count = data["user_info"]["aweme_count"]
        .as_u64()
        .map_or(0, |i|i) as u16;

    let avatar_url = data["user_info"]["avatar_thumb"]["url_list"][0]
        .to_string()
        .replace('"', "");

    Ok(UserInfo{
        nickname,
        uid: uid.to_string(),
        avatar_url,
        video_count,
    })
}  

// 获取用户视频列表
async fn get_user_video_list(uid: String, count: u16, max_cursor: u64) -> Result<VideoInfo> {
    let api_url = format!("https://www.iesdouyin.com/web/api/v2/aweme/post/?sec_uid={uid}&count={count}&max_cursor={max_cursor}");

    let data = reqwest::Client::builder()
        .user_agent(USER_AGNET)
        .build()?
        .get(&api_url)
        .send()
        .await?
        .json::<Value>()
        .await?;

    let max_cursor = data["max_cursor"].as_u64().unwrap_or(0);
    let has_more = data["has_more"].as_bool().unwrap_or(false);

    if !data["aweme_list"].is_array() || data["aweme_list"].as_array().unwrap_or(&Vec::<Value>::new()).is_empty() {
        return Ok(VideoInfo {
            max_cursor,
            has_more,
            items: Vec::<VideoInfoItem>::new(),
        });
    }

    let video_items = data["aweme_list"]
        .as_array()
        .unwrap_or(&Vec::<Value>::new())
        .iter()
        .map(|item| {
            let video_id = item["aweme_id"].to_string().replace('"', "");
            let mut video_title = item["desc"]
                .to_string()
                .replace('"', "")
                .split('#')
                .collect::<Vec<&str>>()[0]
                .to_string()
                .split('@')
                .collect::<Vec<&str>>()[0]
                .to_string();
            if video_title.is_empty() {
                video_title = format!("无标题{}", video_id);
            }
            let video_url = item["video"]["play_addr"]["url_list"][0].to_string().replace('"', "");
            let cover_url = item["video"]["cover"]["url_list"][0].to_string().replace('"', "");
            VideoInfoItem {
                video_id,
                video_title,
                video_url,
                cover_url,
            }
        }).collect::<Vec<VideoInfoItem>>();
    
    Ok(VideoInfo {
        max_cursor,
        has_more,
        items: video_items,
    })
}


// 用户主页视频搜索
#[tauri::command]
pub async fn douyin_muplit_search(home_url: String) -> Result<UserVideoInfo, String>  {

    let client = reqwest::Client::builder()
        .user_agent(USER_AGNET)
        .build()
        .map_err(|_|{ DyError::SystemAbnormal.to_string()})?;
    
    let mut real_url = client.get(&home_url)
        .send()
        .await
        .map_err(|_| DyError::RequestFailed.to_string())?
        .url()
        .to_string();

    let uid = get_id_from_url(&mut real_url);

    if uid.is_empty() {
        return Err(DyError::UserInfoNotFound.to_string()); 
    }
    
    let user_info = get_user_info(&uid)
        .await
        .map_err(|_|{ DyError::GetUserInfoFailed.to_string()})?;

    let video_info = get_user_video_list(uid, user_info.video_count, 0)
        .await
        .map_err(|_| { DyError::VideoInfoNotFound.to_string()})?;
    
    Ok(UserVideoInfo { user_info, video_info })
}


// 查询用户所有视频列表, 并发送事件通知到前端页面
#[tauri::command]
pub async fn douyin_get_all_video_info(uid: String, video_count: u16, max_cursor: u64, window: tauri::Window) -> Result<(), String> {
 
    let mut cursor = max_cursor;
    let mut retry_num = 3;
   
    loop {
        let result = get_user_video_list(uid.clone(), video_count, cursor)
        .await
        .map_err(|_| {DyError::GetVideoInfoFailed.to_string()});

        if let Ok(v_info) = result {
            cursor = v_info.max_cursor;

            let has_more = v_info.has_more;
            
            let _ = window.emit("douyin_get_all_video_info", v_info );

            if !has_more { 
                break;
            }
        }else {
            if retry_num < 0 {
                break;
            }
            retry_num -= 1;
        }
    }
    Ok(())
}

// 拼接保存路径
pub fn get_save_path(save_dir: &String, video_title: &str) -> String {
    let items: Vec<&str> = video_title.split('#').collect();
    let save_path = Path::new(&save_dir).join(&items[0].trim()).to_str().unwrap().to_string() + ".mp4"; 
    save_path
}


// 批量下载视频
#[tauri::command]
pub async fn douyin_muplit_download(items: Vec<VideoInfoItem>, save_dir: String, window: Window) -> Result<(), String>{

    let window = Arc::new(window);
    let mut handler_list = Vec::new();
  
    for item in items.iter() {
        let video_title = Arc::new(item.video_title.clone());
        let save_path = Arc::new(get_save_path(&save_dir, &video_title.clone())); 
        let video_id = Arc::new(item.video_id.clone());
        let result = Downloader::new(item.video_url.clone(), save_path.clone().to_string(), Some(8)).await;
        let window = window.clone();

        if result.is_err() {
            let _ = window.emit("douyin_muplit_download", DyMuplitDownloadProgress { 
                video_id: video_id.to_string(),
                is_success: false,
                video_title: video_title.to_string(),
                save_path: save_path.to_string(),
            });
            continue;
        }

        let downloader = result.ok().unwrap();

        let (tx, mut rx) = mpsc::channel::<bool>(1);
        
        // 下载文件
        tokio::spawn(async move {
            let is_success = downloader.download().await.is_ok();
            let _ = tx.send(is_success).await; // 下载是否成功
        });
        
        // 监听下载进度
        let handler = tokio::spawn(async move {
            loop {
                tokio::select! {
                    val = rx.recv() => {
                        let _ = window.emit("douyin_muplit_download", DyMuplitDownloadProgress { 
                            video_id: video_id.to_string(),
                            is_success: val.unwrap_or(false),
                            video_title: video_title.to_string(),
                            save_path: save_path.to_string(),
                        });
                        break;
                    }
                    _ = sleep(Duration::from_secs(1)) => {}
                }
            }
        });
        handler_list.push(handler);
    }
    join_all(handler_list).await; // 等待监听下载任务完成
    Ok(())
}