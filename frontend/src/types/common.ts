export interface Tag {
  id: string
  name: string
}

export interface Category {
  id: string
  name: string
}

export interface Staff {
  id: string
  name: string
  external_id?: string
  image_url?: string
}

export interface Mylist {
  id: string
  name: string
  item_count: number
  cover_urls: string[]
}

export interface ApiError {
  code: string
  message: string
}
