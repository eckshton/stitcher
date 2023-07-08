use std::env;
use std::io::{Error, Write};
use std::path::PathBuf;
use std::fs;
use std::fs::File;
use std::u32::MAX;
use image::{DynamicImage, ImageError, GenericImageView};
use image::io::Reader;
use image::{ImageBuffer, ColorType};

struct WH {
    w: u32,
    h: u32
}
struct Pt {
    x: u32,
    y: u32
}
struct SPt {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    o: Pt
}
impl Clone for Pt { fn clone(&self) -> Pt { *self } }
impl Clone for SPt { fn clone(&self) -> SPt { *self } }
impl Copy for Pt {}
impl Copy for SPt {}
fn loadimg(path: PathBuf) -> Result<DynamicImage, ImageError> {
    Ok(Reader::open(path).map_err(ImageError::IoError)?.decode()?)
}
fn loadimgdir(path: String) -> Result<Vec<DynamicImage>, ImageError> {
    let mut res: Vec<DynamicImage> = vec![];
    for entry in fs::read_dir(path)? { match loadimg(entry?.path()) {
        Ok(img) => res.push(img),
        Err(..) => {}
    }};
    Ok(res)
}
fn getfilenames(path: String) -> Result<Vec<String>, Error> {
    let mut res: Vec<String> = vec![];
    for entry in fs::read_dir(path)? { match entry?.file_name().to_str() {
        Some(name) => {
            let real = name.to_string();
            match real.find('.') {
                Some(ind) => res.push(real.split_at(ind).0.to_string()),
                None => {}
            }
        },
        None => {}
    }}
    Ok(res)
}
fn scorept(pt: SPt, img: &DynamicImage, dim: &WH) -> u32 {
    if img.width() > pt.w || img.height() > pt.h { return MAX }
    let w = match pt.x + img.width() {
        x if x > dim.w => x,
        x if x <= dim.w => dim.w,
        _ => MAX/2
    };
    let h = match pt.y + img.height() {
        x if x > dim.h => x,
        x if x <= dim.h => dim.h,
        _ => MAX/2
    };
    w+h
}
fn pickpt(pts: &mut Vec<SPt>, upts: &mut Vec<SPt>, img: &DynamicImage, dim: &mut WH) -> Pt {
    let mut rpt = Pt {x:0, y:0};
    let mut low = MAX;
    let mut ind: usize = 0;
    for i in 0..pts.len() { match scorept(pts[i], img, &dim) {
        x if x >= low => {},
        x if x < low => {
            low = x;
            ind = i;
            rpt.x = pts[i].x;
            rpt.y = pts[i].y;
        },
        _ => {}
    }}
    if rpt.x + img.width() > dim.w { dim.w = rpt.x + img.width() }
    if rpt.y + img.height() > dim.h { dim.h = rpt.y + img.height() }
    upts.push(pts[ind].clone());
    pts.swap_remove(ind);
    rpt
}
fn within(pt: u32, test: u32, testo: u32) -> bool {
    if test == testo { return false }
    if pt < testo { return false }
    if pt > test { return false }
    true
}
fn resizept(mut pt: SPt, test: SPt) -> SPt {
    if pt.y > test.y || pt.x > test.x { return pt }
    if within(pt.x, test.x, test.o.x) && pt.h > test.y - pt.y {
        pt.h = test.y - pt.y;
    }
    if within(pt.y, test.y, test.o.y) && pt.w > test.x - pt.x {
        pt.w = test.x - pt.x;
    }
    pt
}
fn addpts(pts: &mut Vec<SPt>, upts: &Vec<SPt>, img: &DynamicImage, pt: Pt) {
    let mut tr = SPt {x:pt.x + img.width(), y:pt.y, w:MAX, h:MAX, o:pt};
    let mut bl = SPt {y:pt.y + img.height(), x:pt.x, w:MAX, h:MAX, o:pt};
    pts.iter_mut().for_each(|pt| {
        tr = resizept(tr, *pt);
        bl = resizept(bl, *pt);
        *pt = resizept(resizept(*pt, tr), bl);
    });
    upts.iter().for_each(|pt| {
        tr = resizept(tr, *pt);
        bl = resizept(bl, *pt);
    });
    pts.push(tr);
    pts.push(bl);
}
fn calcpts(imgs: &Vec<DynamicImage>, res: &mut Vec<Pt>) -> WH {
    let mut pts: Vec<SPt> = vec![SPt {x:0, y:0, w:MAX, h:MAX, o:Pt {x:0, y:0}}];
    let mut upts: Vec<SPt> = vec![];
    let mut dim = WH {w:0, h:0};
    for img in imgs {
        res.push(pickpt(&mut pts, &mut upts, img, &mut dim)); 
        addpts(&mut pts, &upts, img, res[res.len() - 1]); 
    }
    dim
}
fn main() -> Result<(), ImageError> {
    let input: String = match env::args().nth(1) {
        Some(x) => x,
        None => "assets".to_string()
    };
    let imgs: Vec<DynamicImage> = loadimgdir(input.clone())?;
    let imgnames = getfilenames(input.clone())?;
    let mut pts: Vec<Pt> = vec![];
    let mut ind: String = "".to_string();
    let dim = calcpts(&imgs, &mut pts);
    let mut res = ImageBuffer::new(dim.w, dim.h);
    for i in 0..imgs.len() {
        ind.push_str(&format!("{}:{},{}+{}+{}\n", imgnames[i], pts[i].x, pts[i].y, imgs[i].width(), imgs[i].height()).to_string());
        for x in 0..imgs[i].width() { for y in 0..imgs[i].height() {
            res.put_pixel(pts[i].x + x, pts[i].y + y, imgs[i].get_pixel(x, y))
        }}
    }
    image::save_buffer_with_format(format!("{}.png",input), &res, dim.w, dim.h, ColorType::Rgba8, image::ImageFormat::Png)?;
    let mut index = File::create(format!("{}.ref", input))?;
    index.write(ind.as_bytes()).unwrap();
    Ok(())
}
