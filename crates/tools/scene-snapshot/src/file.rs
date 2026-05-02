use std::fs;
use std::path::{Path, PathBuf};

use serde_yaml::Value;

use crate::{SceneSnapshotError, SceneSnapshotImage, SceneSnapshotRequest, SceneSnapshotService};

#[derive(Clone, Debug, Default)]
pub struct FileSceneSnapshotService;

impl SceneSnapshotService for FileSceneSnapshotService {
    fn capture(&self, request: SceneSnapshotRequest) -> Result<SceneSnapshotImage, SceneSnapshotError> {
        if request.width == 0 || request.height == 0 {
            return Err(SceneSnapshotError::new("Snapshot dimensions must be non-zero"));
        }
        let scene_path = request.scene_path.clone().ok_or_else(|| SceneSnapshotError::new("Scene snapshot request is missing scene_path"))?;
        let mod_root = request.mod_root.clone().ok_or_else(|| SceneSnapshotError::new("Scene snapshot request is missing mod_root"))?;
        let source = fs::read_to_string(&scene_path).map_err(|err| SceneSnapshotError::new(format!("Cannot read scene `{}`: {err}", scene_path.display())))?;
        let document = serde_yaml::from_str::<Value>(&source).map_err(|err| SceneSnapshotError::new(format!("Cannot parse scene `{}`: {err}", scene_path.display())))?;
        let entities = document.get("entities").and_then(Value::as_sequence).cloned().unwrap_or_default();
        let mut renderables = collect_renderables(&entities);
        renderables.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap_or(std::cmp::Ordering::Equal));
        let world = WorldMapper::from_scene(request.width, request.height, &renderables);
        let mut surface = Surface::new(request.width, request.height, [0, 0, 0, 255]);

        for renderable in &renderables {
            match &renderable.kind {
                RenderableKind::Sprite { texture, size } => {
                    let screen_size = (size.0 * world.scale, size.1 * world.scale);
                    let center = world.to_screen(renderable.x, renderable.y);
                    if let Some(path) = resolve_texture_path(&mod_root, &request.mod_id, texture) {
                        if surface.blit_image(&path, center, screen_size) {
                            continue;
                        }
                    }
                    surface.draw_rect_center(center, screen_size, [36, 123, 160, 210]);
                }
                RenderableKind::Polygon { points, fill, stroke, stroke_width } => {
                    let points = points.iter().map(|p| world.to_screen(renderable.x + p.0, renderable.y + p.1)).collect::<Vec<_>>();
                    if fill[3] > 0 { surface.fill_polygon(&points, *fill); } if stroke[3] > 0 && *stroke_width > 0.0 { surface.draw_polyline(&points, true, (*stroke_width * world.scale).clamp(1.0, 8.0), *stroke); }
                }
                RenderableKind::Circle { radius, color } => {
                    surface.draw_circle(world.to_screen(renderable.x, renderable.y), radius * world.scale, *color);
                }
                RenderableKind::TileMap { tileset, tile_size, grid } => {
                    surface.draw_tilemap(&mod_root, &request.mod_id, tileset, world.to_screen(renderable.x, renderable.y), world.scale, *tile_size, grid);
                }
                RenderableKind::Text { text, size, color, panel } => {
                    let center = world.to_screen(renderable.x, renderable.y);
                    let screen_size = (size.0 * world.scale, size.1 * world.scale);
                    if let Some(panel_color) = panel {
                        surface.draw_rect_center(center, screen_size, *panel_color);
                    }
                    surface.draw_text_center(center, text, (size.1 * world.scale * 0.52).clamp(8.0, 54.0), *color);
                }
            }
        }

        Ok(SceneSnapshotImage {
            width: request.width,
            height: request.height,
            diagnostic_label: format!("file scene snapshot: {} / {} ({} entities, {} renderables)", request.mod_id, request.scene_id, entities.len(), renderables.len()),
            request,
            pixels_rgba8: surface.pixels,
        })
    }
}

#[derive(Clone, Debug)]
struct Renderable { x: f32, y: f32, z: f32, kind: RenderableKind }

#[derive(Clone, Debug)]
enum RenderableKind {
    Sprite { texture: String, size: (f32, f32) },
    Polygon { points: Vec<(f32, f32)>, fill: [u8; 4], stroke: [u8; 4], stroke_width: f32 },
    Circle { radius: f32, color: [u8; 4] },
    TileMap { tileset: String, tile_size: (f32, f32), grid: Vec<String> },
    Text { text: String, size: (f32, f32), color: [u8; 4], panel: Option<[u8; 4]> },
}

fn collect_renderables(entities: &[Value]) -> Vec<Renderable> {
    let mut renderables = Vec::new();
    for (index, entity) in entities.iter().enumerate() {
        let (x, y) = entity_translation(entity).or_else(|| tilemap_marker_translation(entity, entities)).unwrap_or(fallback_position(index));
        let components = entity.get("components").and_then(Value::as_sequence).cloned().unwrap_or_default();
        for component in components {
            let kind = component.get("type").and_then(Value::as_str).unwrap_or_default();
            match kind {
                "Sprite2D" => {
                    if let Some(texture) = component.get("texture").and_then(Value::as_str) {
                        renderables.push(Renderable { x, y, z: number(&component, "z_index").unwrap_or(0.0), kind: RenderableKind::Sprite { texture: texture.to_string(), size: vector2(&component, "size").unwrap_or((48.0, 48.0)) } });
                    }
                }
                "VectorShape2D" => {
                    if component.get("kind").and_then(Value::as_str) == Some("polygon") {
                        if let Some(points) = polygon_points(&component) {
                            renderables.push(Renderable { x, y, z: number(&component, "z_index").unwrap_or(0.0), kind: RenderableKind::Polygon { points, fill: parse_color(component.get("fill_color").and_then(Value::as_str).unwrap_or("#00000000")), stroke: parse_color(component.get("stroke_color").and_then(Value::as_str).unwrap_or("#123D9AFF")), stroke_width: number(&component, "stroke_width").unwrap_or(1.0) } });
                        }
                    }
                }
                "CircleCollider2D" | "AabbCollider2D" => {
                    let radius = number(&component, "radius").or_else(|| vector2(&component, "size").map(|s| s.0.max(s.1) * 0.5)).unwrap_or(14.0);
                    renderables.push(Renderable { x, y, z: 1000.0, kind: RenderableKind::Circle { radius, color: [255, 22, 84, 180] } });
                }
                "TileMap2D" => {
                    if let Some(grid) = string_sequence(&component, "grid") {
                        renderables.push(Renderable { x, y, z: number(&component, "z_index").unwrap_or(-10.0), kind: RenderableKind::TileMap { tileset: component.get("tileset").and_then(Value::as_str).unwrap_or_default().to_string(), tile_size: vector2(&component, "tile_size").unwrap_or((64.0, 64.0)), grid } });
                    }
                }
                "Text2D" | "BitmapText2D" => {
                    if let Some(text) = component.get("text").and_then(Value::as_str) {
                        renderables.push(Renderable { x, y, z: number(&component, "z_index").unwrap_or(2000.0), kind: RenderableKind::Text { text: text.to_string(), size: vector2(&component, "size").unwrap_or((360.0, 48.0)), color: parse_color(component.get("color").and_then(Value::as_str).unwrap_or("#123D9AFF")), panel: None } });
                    }
                }
                "UiDocument" => collect_ui_document(&component, &mut renderables),
                _ => {}
            }
        }
    }
    renderables
}

fn collect_ui_document(component: &Value, renderables: &mut Vec<Renderable>) {
    let viewport = component.get("target").and_then(|t| t.get("viewport"));
    let viewport_w = viewport.and_then(|v| number(v, "width")).unwrap_or(1280.0);
    let viewport_h = viewport.and_then(|v| number(v, "height")).unwrap_or(720.0);
    if let Some(root) = component.get("root") {
        let mut cursor_y = viewport_h * 0.5 - 86.0;
        collect_ui_node(root, renderables, viewport_w, &mut cursor_y, 2400.0);
    }
}

fn collect_ui_node(node: &Value, renderables: &mut Vec<Renderable>, viewport_w: f32, cursor_y: &mut f32, z: f32) {
    if let Some(children) = node.get("children").and_then(Value::as_sequence) {
        for child in children { collect_ui_node(child, renderables, viewport_w, cursor_y, z + 1.0); }
        return;
    }
    let kind = node.get("type").and_then(Value::as_str).unwrap_or_default();
    let style = node.get("style").unwrap_or(&Value::Null);
    let width = number(style, "width").unwrap_or(if kind == "button" { 520.0 } else { viewport_w * 0.72 });
    let height = number(style, "height").unwrap_or(if kind == "button" { 52.0 } else if kind == "spacer" { 24.0 } else { number(style, "font_size").unwrap_or(20.0) * 1.6 });
    if kind == "spacer" { *cursor_y -= height; return; }
    if let Some(text) = node.get("text").and_then(Value::as_str) {
        let is_primary = node.get("style_class").and_then(Value::as_str) == Some("button_primary");
        let panel = if kind == "button" { Some(if is_primary { [18, 61, 154, 230] } else { [255, 255, 255, 220] }) } else { None };
        let color = if is_primary { [250, 248, 238, 255] } else { parse_color(style.get("color").and_then(Value::as_str).unwrap_or("#123D9AFF")) };
        renderables.push(Renderable { x: 0.0, y: *cursor_y, z, kind: RenderableKind::Text { text: text.to_string(), size: (width, height), color, panel } });
    }
    *cursor_y -= height + 10.0;
}

fn resolve_texture_path(mod_root: &Path, mod_id: &str, key: &str) -> Option<PathBuf> {
    let local = key.strip_prefix(&format!("{mod_id}/")).unwrap_or(key);
    [mod_root.join(local), mod_root.join(format!("{local}.png")), mod_root.join(format!("{local}.jpg")), mod_root.join(format!("{local}.jpeg")), mod_root.join("assets").join(local), mod_root.join("assets").join(format!("{local}.png")), mod_root.join("textures").join(local), mod_root.join("textures").join(format!("{local}.png")), mod_root.join("textures").join(format!("{local}.jpg")), mod_root.join("textures").join(format!("{local}.jpeg")), mod_root.join("tilesets").join(local), mod_root.join("tilesets").join(format!("{local}.png")), mod_root.join("fonts").join(local), mod_root.join("fonts").join(format!("{local}.png"))].into_iter().find(|p| p.is_file())
}

fn resolve_tileset_image(mod_root: &Path, mod_id: &str, key: &str) -> Option<PathBuf> {
    let local = key.strip_prefix(&format!("{mod_id}/")).unwrap_or(key);
    [mod_root.join(format!("{local}.png")), mod_root.join(local).with_extension("png"), mod_root.join("tilesets").join("dirt.png")].into_iter().find(|p| p.is_file())
}

struct WorldMapper { center_x: f32, center_y: f32, scale: f32, width: u32, height: u32 }
impl WorldMapper {
    fn from_scene(width: u32, height: u32, renderables: &[Renderable]) -> Self {
        if should_use_fixed_viewport(renderables) {
            let scale = (width as f32 / 1280.0).min(height as f32 / 720.0);
            return Self { center_x: 0.0, center_y: 0.0, scale, width, height };
        }

        let mut min_x: f32 = -640.0; let mut max_x: f32 = 640.0; let mut min_y: f32 = -360.0; let mut max_y: f32 = 360.0;
        for r in renderables { let (x0,y0,x1,y1)=r.bounds(); min_x=min_x.min(x0); max_x=max_x.max(x1); min_y=min_y.min(y0); max_y=max_y.max(y1); }
        let world_w=(max_x-min_x).max(1.0); let world_h=(max_y-min_y).max(1.0); let scale=(width as f32/world_w).min(height as f32/world_h)*0.94;
        Self { center_x:(min_x+max_x)*0.5, center_y:(min_y+max_y)*0.5, scale, width, height }
    }
    fn to_screen(&self, x:f32, y:f32)->(f32,f32){(self.width as f32*0.5+(x-self.center_x)*self.scale,self.height as f32*0.5-(y-self.center_y)*self.scale)}
}

impl Renderable { fn bounds(&self)->(f32,f32,f32,f32){ match &self.kind { RenderableKind::Sprite{size,..}|RenderableKind::Text{size,..}=>(self.x-size.0*0.5,self.y-size.1*0.5,self.x+size.0*0.5,self.y+size.1*0.5), RenderableKind::Circle{radius,..}=>(self.x-radius,self.y-radius,self.x+radius,self.y+radius), RenderableKind::Polygon{points,..}=>points.iter().fold((self.x,self.y,self.x,self.y),|b,p|(b.0.min(self.x+p.0),b.1.min(self.y+p.1),b.2.max(self.x+p.0),b.3.max(self.y+p.1))), RenderableKind::TileMap{tile_size,grid,..}=>{let rows=grid.len() as f32; let cols=grid.iter().map(|r|r.chars().count()).max().unwrap_or(0) as f32; (self.x-cols*tile_size.0*0.5,self.y-rows*tile_size.1*0.5,self.x+cols*tile_size.0*0.5,self.y+rows*tile_size.1*0.5)}}}}

struct Surface { width:u32, height:u32, pixels:Vec<u8> }
impl Surface {
    fn new(width:u32,height:u32,color:[u8;4])->Self{let mut s=Self{width,height,pixels:vec![0;width as usize*height as usize*4]}; for y in 0..height{for x in 0..width{s.set_pixel(x,y,color)}} s}
    fn blit_image(&mut self,path:&Path,center:(f32,f32),size:(f32,f32))->bool{let Ok(image)=image::open(path) else{return false}; let image=image.to_rgba8(); let (sw,sh)=(image.width(),image.height()); let dw=size.0.abs().round().max(1.0) as u32; let dh=size.1.abs().round().max(1.0) as u32; let l=(center.0-dw as f32*0.5).round() as i32; let t=(center.1-dh as f32*0.5).round() as i32; for y in 0..dh{for x in 0..dw{let sx=(x as f32/dw as f32*sw as f32).floor() as u32; let sy=(y as f32/dh as f32*sh as f32).floor() as u32; self.blend_pixel_i32(l+x as i32,t+y as i32,image.get_pixel(sx.min(sw-1),sy.min(sh-1)).0)}} true}
    fn draw_rect_center(&mut self,center:(f32,f32),size:(f32,f32),color:[u8;4]){let l=(center.0-size.0*0.5).round() as i32; let t=(center.1-size.1*0.5).round() as i32; for y in 0..size.1.abs().round() as i32{for x in 0..size.0.abs().round() as i32{self.blend_pixel_i32(l+x,t+y,color)}}}
    fn draw_circle(&mut self,center:(f32,f32),radius:f32,color:[u8;4]){let r=radius.max(1.0); let l=(center.0-r).floor() as i32; let t=(center.1-r).floor() as i32; let d=(r*2.0).ceil() as i32; for y in 0..d{for x in 0..d{let dx=l+x-center.0.round() as i32; let dy=t+y-center.1.round() as i32; if (dx*dx+dy*dy) as f32<=r*r{self.blend_pixel_i32(l+x,t+y,color)}}}}
    fn draw_tilemap(&mut self,mod_root:&Path,mod_id:&str,tileset:&str,center:(f32,f32),scale:f32,tile_size:(f32,f32),grid:&[String]){let rows=grid.len(); let cols=grid.iter().map(|r|r.chars().count()).max().unwrap_or(0); if rows==0||cols==0{return}; let tw=(tile_size.0*scale).abs().max(1.0); let th=(tile_size.1*scale).abs().max(1.0); let left=center.0-cols as f32*tw*0.5; let top=center.1-rows as f32*th*0.5; let atlas=resolve_tileset_image(mod_root,mod_id,tileset).and_then(|p|image::open(p).ok()).map(|i|i.to_rgba8()); for (ri,row) in grid.iter().enumerate(){for (ci,symbol) in row.chars().enumerate(){if symbol=='.'||symbol==' '{continue} let id=tile_id_for_symbol(symbol); let x=left+ci as f32*tw+tw*0.5; let y=top+ri as f32*th+th*0.5; if let Some(a)=&atlas{self.blit_tile(a,id,(x,y),(tw,th))}else{self.draw_rect_center((x,y),(tw,th),[36,123,160,210])}}}}
    fn blit_tile(&mut self,atlas:&image::RgbaImage,tile_id:u32,center:(f32,f32),size:(f32,f32)){let ts=256; let sx0=(tile_id%8)*ts; let sy0=(tile_id/8)*ts; let dw=size.0.round().max(1.0) as u32; let dh=size.1.round().max(1.0) as u32; let l=(center.0-dw as f32*0.5).round() as i32; let t=(center.1-dh as f32*0.5).round() as i32; for y in 0..dh{for x in 0..dw{let sx=sx0+(x as f32/dw as f32*ts as f32).floor() as u32; let sy=sy0+(y as f32/dh as f32*ts as f32).floor() as u32; if sx<atlas.width()&&sy<atlas.height(){self.blend_pixel_i32(l+x as i32,t+y as i32,atlas.get_pixel(sx,sy).0)}}}}
    fn draw_text_center(&mut self,center:(f32,f32),text:&str,size:f32,color:[u8;4]){let scale=(size/7.0).max(1.0).round() as i32; let char_w=6*scale; let width=text.chars().count() as i32*char_w; let mut x=center.0.round() as i32-width/2; let y=center.1.round() as i32-(7*scale)/2; for ch in text.chars(){self.draw_char(x,y,scale,ch,color); x+=char_w}}
    fn draw_char(&mut self,x:i32,y:i32,scale:i32,ch:char,color:[u8;4]){let glyph=glyph(ch); for (row,bits) in glyph.iter().enumerate(){for col in 0..5{if bits & (1<<(4-col)) !=0{for sy in 0..scale{for sx in 0..scale{self.blend_pixel_i32(x+col*scale+sx,y+row as i32*scale+sy,color)}}}}}}
    fn fill_polygon(&mut self,points:&[(f32,f32)],color:[u8;4]){if points.len()<3{return}; let min_x=points.iter().map(|p|p.0).fold(f32::INFINITY,f32::min).floor().max(0.0) as i32; let max_x=points.iter().map(|p|p.0).fold(f32::NEG_INFINITY,f32::max).ceil().min(self.width as f32) as i32; let min_y=points.iter().map(|p|p.1).fold(f32::INFINITY,f32::min).floor().max(0.0) as i32; let max_y=points.iter().map(|p|p.1).fold(f32::NEG_INFINITY,f32::max).ceil().min(self.height as f32) as i32; for y in min_y..max_y{for x in min_x..max_x{if point_in_polygon(x as f32+0.5,y as f32+0.5,points){self.blend_pixel_i32(x,y,color)}}}}
    fn draw_polyline(&mut self,points:&[(f32,f32)],closed:bool,width:f32,color:[u8;4]){if points.len()<2{return}; for pair in points.windows(2){self.draw_line(pair[0],pair[1],width,color)} if closed{self.draw_line(*points.last().unwrap(),points[0],width,color)}}
    fn draw_line(&mut self,a:(f32,f32),b:(f32,f32),width:f32,color:[u8;4]){let dx=b.0-a.0; let dy=b.1-a.1; let steps=dx.abs().max(dy.abs()).ceil().max(1.0) as i32; let radius=(width*0.5).ceil().max(1.0) as i32; for step in 0..=steps{let t=step as f32/steps as f32; let x=(a.0+dx*t).round() as i32; let y=(a.1+dy*t).round() as i32; for oy in -radius..=radius{for ox in -radius..=radius{if (ox*ox+oy*oy) as f32 <= radius as f32*radius as f32{self.blend_pixel_i32(x+ox,y+oy,color)}}}}}
    fn set_pixel(&mut self,x:u32,y:u32,color:[u8;4]){let o=(y as usize*self.width as usize+x as usize)*4; self.pixels[o..o+4].copy_from_slice(&color)}
    fn blend_pixel_i32(&mut self,x:i32,y:i32,color:[u8;4]){if x<0||y<0||x>=self.width as i32||y>=self.height as i32{return}; let o=(y as usize*self.width as usize+x as usize)*4; let a=color[3] as f32/255.0; self.pixels[o]=blend(self.pixels[o],color[0],a); self.pixels[o+1]=blend(self.pixels[o+1],color[1],a); self.pixels[o+2]=blend(self.pixels[o+2],color[2],a); self.pixels[o+3]=255}
}

fn should_use_fixed_viewport(renderables:&[Renderable])->bool{let has_ui=renderables.iter().any(|r|matches!(&r.kind,RenderableKind::Text{..})&&r.z>=2000.0); let has_paper=renderables.iter().any(|r|matches!(&r.kind,RenderableKind::Sprite{size,..} if size.0>=1200.0&&size.1>=700.0)); let has_huge_tilemap=renderables.iter().any(|r|matches!(&r.kind,RenderableKind::TileMap{tile_size,grid,..} if grid.iter().map(|row|row.chars().count()).max().unwrap_or(0) as f32*tile_size.0>2500.0)); (has_ui||has_paper)&&!has_huge_tilemap}
fn tile_id_for_symbol(symbol:char)->u32{match symbol{'#'=>4,'='=>45,'P'=>46,'C'=>61,'F'=>47,'/'=>40,'\\'=>42,'_'=>8,'*'=>53,_=>0}}
fn entity_matches(entity:&Value,reference:&str)->bool{entity.get("id").and_then(Value::as_str)==Some(reference)||entity.get("name").and_then(Value::as_str)==Some(reference)}
fn tilemap_component(entity:&Value)->Option<&Value>{entity.get("components").and_then(Value::as_sequence)?.iter().find(|component|component.get("type").and_then(Value::as_str)==Some("TileMap2D"))}
fn tilemap_marker_translation(entity:&Value,entities:&[Value])->Option<(f32,f32)>{let marker=entity.get("components").and_then(Value::as_sequence)?.iter().find(|component|component.get("type").and_then(Value::as_str)==Some("TileMapMarker2D"))?; let tilemap_ref=marker.get("tilemap_entity").and_then(Value::as_str)?; let symbol=marker.get("symbol").and_then(Value::as_str)?.chars().next()?; let wanted_index=marker.get("index").and_then(Value::as_u64).unwrap_or(0) as usize; let offset=vector2(marker,"offset").unwrap_or((0.0,0.0)); let tilemap_entity=entities.iter().find(|candidate|entity_matches(candidate,tilemap_ref))?; let (base_x,base_y)=entity_translation(tilemap_entity).unwrap_or((0.0,0.0)); let tilemap=tilemap_component(tilemap_entity)?; let tile_size=vector2(tilemap,"tile_size").unwrap_or((64.0,64.0)); let grid=string_sequence(tilemap,"grid")?; let rows=grid.len(); let cols=grid.iter().map(|row|row.chars().count()).max().unwrap_or(0); let mut seen=0usize; for (row_index,row) in grid.iter().enumerate(){for (column_index,cell) in row.chars().enumerate(){if cell!=symbol{continue} if seen==wanted_index{let x=base_x-cols as f32*tile_size.0*0.5+column_index as f32*tile_size.0+tile_size.0*0.5+offset.0; let y=base_y+rows as f32*tile_size.1*0.5-row_index as f32*tile_size.1-tile_size.1*0.5+offset.1; return Some((x,y));} seen+=1;}} None}
fn point_in_polygon(x:f32,y:f32,points:&[(f32,f32)])->bool{let mut inside=false; let mut j=points.len()-1; for i in 0..points.len(){let(xi,yi)=points[i]; let(xj,yj)=points[j]; if ((yi>y)!=(yj>y))&&(x<(xj-xi)*(y-yi)/((yj-yi).max(0.0001))+xi){inside=!inside} j=i} inside}
fn entity_translation(entity:&Value)->Option<(f32,f32)>{let t=entity.get("transform2")?.get("translation")?; Some((number(t,"x")?,number(t,"y")?))}
fn vector2(value:&Value,key:&str)->Option<(f32,f32)>{let v=value.get(key)?; Some((number(v,"x")?,number(v,"y")?))}
fn polygon_points(component:&Value)->Option<Vec<(f32,f32)>>{Some(component.get("points")?.as_sequence()?.iter().filter_map(|p|Some((number(p,"x")?,number(p,"y")?))).collect())}
fn string_sequence(value:&Value,key:&str)->Option<Vec<String>>{Some(value.get(key)?.as_sequence()?.iter().filter_map(Value::as_str).map(String::from).collect())}
fn number(value:&Value,key:&str)->Option<f32>{value.get(key).and_then(Value::as_f64).map(|v|v as f32).or_else(||value.get(key).and_then(Value::as_i64).map(|v|v as f32)).or_else(||value.get(key).and_then(Value::as_u64).map(|v|v as f32))}
fn parse_color(hex:&str)->[u8;4]{let h=hex.trim_start_matches('#'); if h.len()!=8&&h.len()!=6{return[36,123,160,180]} let r=u8::from_str_radix(&h[0..2],16).unwrap_or(36); let g=u8::from_str_radix(&h[2..4],16).unwrap_or(123); let b=u8::from_str_radix(&h[4..6],16).unwrap_or(160); let a=if h.len()==8{u8::from_str_radix(&h[6..8],16).unwrap_or(180)}else{255}; [r,g,b,a]}
fn fallback_position(index:usize)->(f32,f32){((index%8)as f32*96.0-336.0,(index/8)as f32*72.0-220.0)}
fn blend(dst:u8,src:u8,alpha:f32)->u8{(dst as f32*(1.0-alpha)+src as f32*alpha).round() as u8}
fn glyph(ch:char)->[u8;7]{match ch.to_ascii_uppercase(){'A'=>[14,17,17,31,17,17,17],'B'=>[30,17,17,30,17,17,30],'C'=>[15,16,16,16,16,16,15],'D'=>[30,17,17,17,17,17,30],'E'=>[31,16,16,30,16,16,31],'F'=>[31,16,16,30,16,16,16],'G'=>[15,16,16,23,17,17,15],'H'=>[17,17,17,31,17,17,17],'I'=>[31,4,4,4,4,4,31],'J'=>[7,2,2,2,18,18,12],'K'=>[17,18,20,24,20,18,17],'L'=>[16,16,16,16,16,16,31],'M'=>[17,27,21,21,17,17,17],'N'=>[17,25,21,19,17,17,17],'O'=>[14,17,17,17,17,17,14],'P'=>[30,17,17,30,16,16,16],'Q'=>[14,17,17,17,21,18,13],'R'=>[30,17,17,30,20,18,17],'S'=>[15,16,16,14,1,1,30],'T'=>[31,4,4,4,4,4,4],'U'=>[17,17,17,17,17,17,14],'V'=>[17,17,17,17,17,10,4],'W'=>[17,17,17,21,21,21,10],'X'=>[17,17,10,4,10,17,17],'Y'=>[17,17,10,4,4,4,4],'Z'=>[31,1,2,4,8,16,31],'0'=>[14,19,21,21,25,17,14],'1'=>[4,12,4,4,4,4,14],'2'=>[14,17,1,2,4,8,31],'3'=>[30,1,1,14,1,1,30],'4'=>[2,6,10,18,31,2,2],'5'=>[31,16,16,30,1,1,30],'6'=>[14,16,16,30,17,17,14],'7'=>[31,1,2,4,8,8,8],'8'=>[14,17,17,14,17,17,14],'9'=>[14,17,17,15,1,1,14],'-'=>[0,0,0,31,0,0,0],':'=>[0,4,0,0,4,0,0],'.'=>[0,0,0,0,0,12,12],_=>[0,0,0,0,0,0,0]}}







