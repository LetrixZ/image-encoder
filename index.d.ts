/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export const enum Format {
  Webp = 0,
  Jpeg = 1,
  Png = 2,
  Jxl = 3,
  Avif = 4
}
export interface EncodeOptions {
  width: number
  format: string
  quality?: number
  speed?: number
  lossless?: boolean
}
export interface ArchiveImage {
  filename: string
  savePath: string
  options: EncodeOptions
}
export interface EncodedImage {
  path: string
  contents: Uint8Array
  width: number
  height: number
}
function encodeImage(path: string, image: ArchiveImage): EncodedImage
function generateImages(path: string, images: Array<ArchiveImage>, threads: number): Array<EncodedImage>
export interface ImagesBatch {
  path: string
  images: Array<ArchiveImage>
}
function generateImagesBatch(batches: Array<ImagesBatch>): Array<EncodedImage>
