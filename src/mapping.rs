use crate::models::{MovieItem, Section};
use serde_json::Value;

pub fn map_item(src: &Value) -> MovieItem {
    let item = if let Some(subj) = src.get("subject") {
        if subj.is_object() {
            subj
        } else {
            src
        }
    } else {
        src
    };

    let mut sid = item
        .get("subjectId")
        .or_else(|| item.get("id"))
        .and_then(|v| {
            if let Some(s) = v.as_str() {
                Some(s.to_string())
            } else if let Some(n) = v.as_i64() {
                Some(n.to_string())
            } else {
                None
            }
        })
        .unwrap_or_default();

    let title = item
        .get("title")
        .or_else(|| item.get("name"))
        .or_else(|| item.get("subjectName"))
        .or_else(|| item.get("subject_name"))
        .or_else(|| item.get("categoryName"))
        .or_else(|| item.get("content"))
        .or_else(|| item.get("keyword"))
        .or_else(|| item.get("keywordName"))
        .or_else(|| item.get("itemName"))
        .or_else(|| item.get("show_name"))
        .or_else(|| item.get("showTitle"))
        .or_else(|| item.get("titleName"))
        .or_else(|| item.get("title_en"))
        .or_else(|| item.get("tag"))
        .or_else(|| item.get("label"))
        .or_else(|| item.get("tabName"))
        .or_else(|| src.get("title"))
        .or_else(|| src.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();

    let dlink = item
        .get("deepLink")
        .or_else(|| src.get("deepLink"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let mut action_type = "movie".to_string();
    let mut category_id: Option<String> = None;

    if !dlink.is_empty() {
        if dlink.contains("/home/category") {
            action_type = "category".to_string();
            if let Some(idx) = dlink.find("categoryType=") {
                let rest = &dlink[idx + 13..];
                let cat = rest.split('&').next().unwrap_or("");
                category_id = Some(cat.to_string());
                if sid.is_empty() || sid == "0" {
                    sid = cat.to_string();
                }
            }
        } else if dlink.contains("/playlist/detail") {
            action_type = "playlist".to_string();
            if let Some(idx) = dlink.find("playlistId=") {
                let rest = &dlink[idx + 11..];
                let pid = rest.split('&').next().unwrap_or("");
                if sid.is_empty() || sid == "0" {
                    sid = pid.to_string();
                }
            }
        } else if dlink.contains("/movie/detail") {
            action_type = "movie".to_string();
            if let Some(idx) = dlink.find("subjectId=") {
                let rest = &dlink[idx + 10..];
                let mid = rest.split('&').next().unwrap_or("");
                if sid.is_empty() || sid == "0" {
                    sid = mid.to_string();
                }
            }
        } else if dlink.contains("/post/") || dlink.contains("detailVideo") {
            action_type = "post".to_string();
            if let Some(idx) = dlink.find("id=") {
                let rest = &dlink[idx + 3..];
                let pid = rest.split('&').next().unwrap_or("");
                if sid.is_empty() || sid == "0" {
                    sid = pid.to_string();
                }
            }
        }
    }

    let mut poster_url = String::new();
    if let Some(poster) = item.get("poster") {
        if let Some(u) = poster.get("url").and_then(|v| v.as_str()) {
            poster_url = u.to_string();
        } else if let Some(s) = poster.as_str() {
            poster_url = s.to_string();
        }
    }

    if poster_url.is_empty() {
        if let Some(cover) = item.get("cover") {
            if let Some(u) = cover.get("url").and_then(|v| v.as_str()) {
                poster_url = u.to_string();
            } else if let Some(s) = cover.as_str() {
                poster_url = s.to_string();
            }
        }
    }

    if poster_url.is_empty() {
        let img_terms = ["image", "img", "thumb", "thumbnail", "poster", "cover", "icon", "banner", "pic", "picture"];
        for term in &img_terms {
            if let Some(val) = item.get(*term) {
                if let Some(u) = val.get("url").and_then(|v| v.as_str()) {
                    poster_url = u.to_string();
                    break;
                } else if let Some(s) = val.as_str() {
                    if s.starts_with("http") || s.starts_with("//") {
                        poster_url = s.to_string();
                        break;
                    }
                }
            }
        }
    }

    if poster_url.is_empty() {
        if let Some(hp) = item.get("horizontalPoster").or_else(|| item.get("horizontalCover")) {
            if let Some(u) = hp.get("url").and_then(|v| v.as_str()) {
                poster_url = u.to_string();
            } else if let Some(s) = hp.as_str() {
                poster_url = s.to_string();
            }
        }
    }

    if poster_url.is_empty() {
        if let Some(banner) = item.get("banner") {
            if let Some(u) = banner.get("image").and_then(|i| i.get("url")).and_then(|v| v.as_str()) {
                poster_url = u.to_string();
            } else if let Some(u) = banner.get("url").and_then(|v| v.as_str()) {
                poster_url = u.to_string();
            }
        }
    }

    if poster_url.starts_with("//") {
        poster_url = format!("https:{}", poster_url);
    }

    let score = item
        .get("imdbRatingValue")
        .or_else(|| item.get("imdbRate"))
        .or_else(|| item.get("starRating"))
        .or_else(|| item.get("score"))
        .map(|v| {
            if let Some(s) = v.as_str() {
                s.to_string()
            } else if let Some(n) = v.as_f64() {
                format!("{:.1}", n)
            } else {
                "N/A".to_string()
            }
        })
        .unwrap_or_else(|| "N/A".to_string());

    let release_date = item
        .get("releaseDate")
        .or_else(|| item.get("releaseTime"))
        .or_else(|| item.get("year"))
        .and_then(|v| {
            if let Some(s) = v.as_str() {
                Some(s.to_string())
            } else if let Some(n) = v.as_i64() {
                Some(n.to_string())
            } else {
                None
            }
        })
        .unwrap_or_default();

    let display_year = if release_date.len() >= 4 {
        release_date[..4].to_string()
    } else {
        "N/A".to_string()
    };

    let runtime = item
        .get("duration")
        .or_else(|| item.get("runtime"))
        .or_else(|| item.get("minute"))
        .map(|v| {
            if let Some(n) = v.as_i64() {
                format!("{}m", n)
            } else if let Some(s) = v.as_str() {
                s.to_string()
            } else {
                String::new()
            }
        })
        .filter(|s| !s.is_empty());

    let season = item.get("season").and_then(|v| v.as_i64());
    let episode = item.get("episode").or_else(|| item.get("ep")).and_then(|v| v.as_i64());
    let see_time = item.get("seeTime").and_then(|v| v.as_i64());
    let seen_status = item.get("seenStatus").and_then(|v| v.as_i64());

    let subject_type = item
        .get("subjectType")
        .or_else(|| item.get("type"))
        .or_else(|| item.get("subject_type"))
        .and_then(|v| v.as_u64())
        .map(|n| n as u32)
        .unwrap_or_else(|| {
            if item.get("episodeCount").is_some() || item.get("seasonCount").is_some() {
                2
            } else {
                1
            }
        });

    let like_status = if item.get("isFavorite").and_then(|v| v.as_i64()) == Some(1)
        || item.get("is_favorite").and_then(|v| v.as_i64()) == Some(1)
        || item.get("isLike").and_then(|v| v.as_i64()) == Some(1)
        || item.get("wantToSee").and_then(|v| v.as_i64()) == Some(1)
        || item.get("isFavorite").and_then(|v| v.as_bool()) == Some(true)
    {
        1
    } else {
        0
    };

    let description = item
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let stream_url = item
        .get("playUrl")
        .and_then(|p| p.get("playUrl"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let is_collection = item.get("isCollection").and_then(|v| v.as_bool()).unwrap_or(false);
    let items = item
        .get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    MovieItem {
        subject_id: sid.clone(),
        id: sid,
        title,
        cover: poster_url.clone(),
        poster: poster_url,
        score,
        release_time: display_year,
        subject_type,
        runtime,
        season,
        episode,
        see_time,
        seen_status,
        like_status,
        description,
        action_type,
        category_id,
        deep_link: dlink.to_string(),
        is_post: false,
        stream_url,
        is_collection,
        items,
    }
}

pub fn format_tab_sections(items: &[Value]) -> Vec<Section> {
    let mut is_direct_movies = true;
    for row in items {
        if row.get("list").is_some()
            || row.get("items").is_some()
            || row.get("subjects").is_some()
            || row.get("movieList").is_some()
            || row.get("customData").is_some()
            || row.get("banner").is_some()
        {
            is_direct_movies = false;
            break;
        }
    }

    if is_direct_movies && !items.is_empty() {
        let mapped: Vec<MovieItem> = items
            .iter()
            .filter(|m| m.get("subjectId").is_some() || m.get("id").is_some())
            .map(map_item)
            .collect();
        if !mapped.is_empty() {
            return vec![Section {
                title: "Content".to_string(),
                items: mapped,
            }];
        }
    }

    let mut sections = Vec::new();

    for row in items {
        let title = row
            .get("title")
            .or_else(|| row.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("Section")
            .to_string();

        let inner: Vec<Value> = if let Some(l) = row.get("list").and_then(|v| v.as_array()) {
            l.clone()
        } else if let Some(it) = row.get("items").and_then(|v| v.as_array()) {
            it.clone()
        } else if let Some(subjs) = row.get("subjects").and_then(|v| v.as_array()) {
            subjs.clone()
        } else if let Some(ml) = row.get("movieList").and_then(|v| v.as_array()) {
            ml.clone()
        } else if let Some(cd) = row.get("customData").and_then(|v| v.get("items")).and_then(|v| v.as_array()) {
            cd.clone()
        } else if let Some(b) = row.get("banner").and_then(|v| v.get("banners")).and_then(|v| v.as_array()) {
            b.clone()
        } else {
            Vec::new()
        };

        let mut real_movies = Vec::new();
        for i in &inner {
            if let Some(subj) = i.get("subject") {
                real_movies.push(map_item(subj));
            } else if i.get("subjectId").is_some() || i.get("id").is_some() {
                real_movies.push(map_item(i));
            }
        }

        if !real_movies.is_empty() {
            sections.push(Section {
                title,
                items: real_movies,
            });
        }
    }

    sections
}
