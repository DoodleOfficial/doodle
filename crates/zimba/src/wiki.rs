/**
 * @file wiki.rs
 * @author Krisna Pranav
 * @brief zimba
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */
use std::collections::HashMap;

use crate::{Cluster, DirEntry, Error, ZimFile};

struct ArticleRef {
    blob_number: u32,
    url: String,
    title: String,
}

struct WorkingCluster<T> {
    cluster: Cluster,
    data: Vec<T>,
}

pub struct ArticleIterator<'a> {
    zim: &'a ZimFile,
    articles: Vec<(u64, Vec<ArticleRef>)>,

    cur_cluster: Option<WorkingCluster<ArticleRef>>,
}

impl<'a> ArticleIterator<'a> {
    pub fn new(zim: &'a ZimFile) -> Result<ArticleIterator<'a>, Error> {
        let mut articles = HashMap::new();
        for entry in zim.dir_entries() {
            let entry = entry?;

            if let DirEntry::Content {
                mime_type,
                parameter_len: _,
                namespace,
                revision: _,
                cluster_number,
                blob_number,
                url,
                title,
            } = entry
            {
                if namespace != 'A' || zim.mime_types()[mime_type] != "text/html" {
                    continue;
                }

                let article_ref = ArticleRef {
                    blob_number,
                    url,
                    title,
                };

                let key = u64::from(cluster_number);

                articles.entry(key).or_insert(Vec::new()).push(article_ref);
            }
        }

        let mut articles = articles
            .into_iter()
            .map(|(cluster, mut articles)| {
                articles.sort_by_key(|article| article.blob_number);
                (cluster, articles)
            })
            .collect::<Vec<_>>();
        articles.sort_by_key(|(cluster_number, _)| *cluster_number);

        Ok(ArticleIterator {
            zim,
            articles,
            cur_cluster: None,
        })
    }
}

#[derive(Debug)]
pub struct Article {
    pub url: String,
    pub title: String,
    pub content: String,
}

impl Iterator for ArticleIterator<'_> {
    type Item = Article;

    fn next(&mut self) -> Option<Self::Item> {
        if self.articles.is_empty() {
            return None;
        }

        if self.cur_cluster.is_none() {
            let (cluster_number, articles) = self.articles.swap_remove(0);
            let cluster = self.zim.get_cluster(cluster_number as u32).ok()??;
            self.cur_cluster = Some(WorkingCluster {
                cluster,
                data: articles,
            });
        }

        let cur_cluster = self.cur_cluster.as_mut()?;

        let article_ref = cur_cluster.data.pop()?;

        let blob = cur_cluster
            .cluster
            .get_blob(article_ref.blob_number as usize)?;

        let mut title = article_ref.title;

        if title.is_empty() {
            title.clone_from(&article_ref.url);
        }

        let article = Article {
            title,
            url: article_ref.url,
            content: String::from_utf8_lossy(blob).to_string(),
        };

        if cur_cluster.data.is_empty() {
            self.cur_cluster = None;
        }

        Some(article)
    }
}

struct ImageRef {
    mime_type: String,
    blob_number: u32,
    url: String,
}

pub struct ImageIterator<'a> {
    zim: &'a ZimFile,
    images: Vec<(u64, Vec<ImageRef>)>,

    cur_cluster: Option<WorkingCluster<ImageRef>>,
}

impl<'a> ImageIterator<'a> {
    pub fn new(zim: &'a ZimFile) -> Result<ImageIterator<'a>, Error> {
        let mut images = HashMap::new();
        for entry in zim.dir_entries() {
            let entry = entry?;

            if let DirEntry::Content {
                mime_type,
                parameter_len: _,
                namespace,
                revision: _,
                cluster_number,
                blob_number,
                url,
                title: _,
            } = entry
            {
                if namespace != 'I' {
                    continue;
                }

                let image_ref = ImageRef {
                    blob_number,
                    url,
                    mime_type: zim.mime_types()[mime_type].clone(),
                };

                let key = u64::from(cluster_number);

                images.entry(key).or_insert(Vec::new()).push(image_ref);
            }
        }

        let mut images = images
            .into_iter()
            .map(|(cluster, mut images)| {
                images.sort_by_key(|image| image.blob_number);
                (cluster, images)
            })
            .collect::<Vec<_>>();
        images.sort_by_key(|(cluster_number, _)| *cluster_number);

        Ok(ImageIterator {
            zim,
            images,
            cur_cluster: None,
        })
    }
}

#[derive(Debug)]
pub struct Image {
    pub url: String,
    pub mime_type: String,
    pub content: Vec<u8>,
}
impl Image {
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.content
    }
}

impl Iterator for ImageIterator<'_> {
    type Item = Image;

    fn next(&mut self) -> Option<Self::Item> {
        if self.images.is_empty() {
            return None;
        }

        if self.cur_cluster.is_none() {
            let (cluster_number, images) = self.images.swap_remove(0);
            let cluster = self.zim.get_cluster(cluster_number as u32).ok()??;
            self.cur_cluster = Some(WorkingCluster {
                cluster,
                data: images,
            });
        }

        let cur_cluster = self.cur_cluster.as_mut()?;

        let image_ref = cur_cluster.data.pop()?;

        let blob = cur_cluster
            .cluster
            .get_blob(image_ref.blob_number as usize)?;

        let image = Image {
            url: image_ref.url,
            mime_type: image_ref.mime_type,
            content: blob.to_vec(),
        };

        if cur_cluster.data.is_empty() {
            self.cur_cluster = None;
        }

        Some(image)
    }
}
