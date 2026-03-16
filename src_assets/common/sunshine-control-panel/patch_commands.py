import re
import os

path = r'd:\项目文件\code\galremote\src_assets\common\sunshine-control-panel\src-tauri\src\galgame\commands.rs'

if not os.path.exists(path):
    print(f"ERROR: File not found: {path}")
    exit(1)

with open(path, 'r', encoding='utf-8') as f:
    content = f.read()

# Define the new merge logic for metadata
metadata_merge_code = """            // [Phase 4] Merge Status
            let merged_status = merge_game_status(
                &local_game.status,
                &cloud_game.status,
                local_game.last_played,
                cloud_game.last_played,
            );
            if merged_status != local_game.status {
                local_game.status = merged_status;
                modified = true;
            }

            // [Phase 4] Merge metadata fields
            if local_game.original_name.is_none() && cloud_game.original_name.is_some() {
                local_game.original_name = cloud_game.original_name.clone();
                modified = true;
            }
            if local_game.sort_name.is_none() && cloud_game.sort_name.is_some() {
                local_game.sort_name = cloud_game.sort_name.clone();
                modified = true;
            }
            if local_game.description.is_none() && cloud_game.description.is_some() {
                local_game.description = cloud_game.description.clone();
                modified = true;
            }
            if local_game.developer.is_none() && cloud_game.developer.is_some() {
                local_game.developer = cloud_game.developer.clone();
                modified = true;
            }
            if local_game.developers.is_empty() && !cloud_game.developers.is_empty() {
                local_game.developers = cloud_game.developers.clone();
                modified = true;
            }
            if local_game.publishers.is_empty() && !cloud_game.publishers.is_empty() {
                local_game.publishers = cloud_game.publishers.clone();
                modified = true;
            }
            if local_game.release_date.is_none() && cloud_game.release_date.is_some() {
                local_game.release_date = cloud_game.release_date.clone();
                modified = true;
            }
            if local_game.genres.is_empty() && !cloud_game.genres.is_empty() {
                local_game.genres = cloud_game.genres.clone();
                modified = true;
            }
            if local_game.tags.is_empty() && !cloud_game.tags.is_empty() {
                local_game.tags = cloud_game.tags.clone();
                modified = true;
            }
            if local_game.platforms.is_empty() && !cloud_game.platforms.is_empty() {
                local_game.platforms = cloud_game.platforms.clone();
                modified = true;
            }

            // [Phase 4] Merge IDs
            if local_game.steam_id.is_none() && cloud_game.steam_id.is_some() {
                local_game.steam_id = cloud_game.steam_id.clone();
                modified = true;
            }
            if local_game.vndb_id.is_none() && cloud_game.vndb_id.is_some() {
                local_game.vndb_id = cloud_game.vndb_id.clone();
                modified = true;
            }
            if local_game.igdb_id.is_none() && cloud_game.igdb_id.is_some() {
                local_game.igdb_id = cloud_game.igdb_id.clone();
                modified = true;
            }
            if local_game.ymgal_id.is_none() && cloud_game.ymgal_id.is_some() {
                local_game.ymgal_id = cloud_game.ymgal_id.clone();
                modified = true;
            }
            if local_game.bangumi_id.is_none() && cloud_game.bangumi_id.is_some() {
                local_game.bangumi_id = cloud_game.bangumi_id.clone();
                modified = true;
            }

            // Merge scores/ratings
            if local_game.score.is_none() && cloud_game.score.is_some() {
                local_game.score = cloud_game.score;
                modified = true;
            }
            if local_game.rating.is_none() && cloud_game.rating.is_some() {
                local_game.rating = cloud_game.rating;
                modified = true;
            }
            
            // Merge images
            if local_game.cover_image.is_none() && cloud_game.cover_image.is_some() {
                local_game.cover_image = cloud_game.cover_image.clone();
                modified = true;
            }
            if local_game.background_image.is_none() && cloud_game.background_image.is_some() {
                local_game.background_image = cloud_game.background_image.clone();
                modified = true;
            }
            if local_game.logo_image.is_none() && cloud_game.logo_image.is_some() {
                local_game.logo_image = cloud_game.logo_image.clone();
                modified = true;
            }"""

# Try matching the whole block
# We look for the part starting from "// Merge Metadata if missing locally" until the end of the status merge block.
# Using a simpler regex that is less sensitive to exact status merge content
pattern = r'// Merge Metadata if missing locally.*?if merged_status != local_game\.status \{.*?\}'

# Perform replacement
new_content = re.sub(pattern, metadata_merge_code, content, flags=re.DOTALL)

if new_content == content:
    print("FAILED: Regex did not match anything.")
else:
    with open(path, 'w', encoding='utf-8', newline='') as f:
        f.write(new_content)
    print("SUCCESS: Patched commands.rs.")
