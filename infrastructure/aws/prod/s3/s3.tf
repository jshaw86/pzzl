resource "aws_s3_bucket" "media" {
  bucket = var.bucket_name 
}

resource "aws_s3_bucket_ownership_controls" "media" {
  bucket = aws_s3_bucket.media.id
  rule {
    object_ownership = "BucketOwnerPreferred"
  }
}

resource "aws_s3_bucket_public_access_block" "media" {
  bucket = aws_s3_bucket.media.id

  block_public_acls       = false
  block_public_policy     = false
  ignore_public_acls      = false
  restrict_public_buckets = false
}

resource "aws_s3_bucket_acl" "media" {
  depends_on = [
    aws_s3_bucket_ownership_controls.media,
    aws_s3_bucket_public_access_block.media,
  ]

  bucket = aws_s3_bucket.media.id
  acl    = "public-read"
}
