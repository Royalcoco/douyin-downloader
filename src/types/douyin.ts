export type UserInfo = {
    nickname: string,
    uid: string,
    avatar_url: string,
    video_count: number,
  }
  
  export type VideoInfoItem = {
  video_id: string,   
  video_title: string, 
  video_url: string,  
  cover_url: string, 
  }
  
  export type VideoInfo = {
  max_cursor: number,
  has_more: boolean,
  items: VideoInfoItem[],
  }
  
  export type UserVideoInfo = {
    user_info: UserInfo,
    video_info: VideoInfo,
  }