#![feature(proc_macro_hygiene)]
#![feature(decl_macro)]
mod config;
#[macro_use]
extern crate rocket;


use std::io;
use std::env;
use rocket::{Request, Handler, Route, Data, Catcher, Response};
use rocket::http::{Status, RawStr};
use rocket::response::{self, Responder, status::Custom};
use rocket::handler::Outcome;
use rocket::outcome::IntoOutcome;
use rocket::http::Method::*;
use std::fs::File;

fn main() {
    //rocket::ignite().launch();
    rocket().launch();
}
#[derive(Clone)]
struct CustomHandler{
    data:&'static str
}
impl CustomHandler{
    fn new(data:&'static str)->Vec<Route>{
        vec![Route::new(Get,"/<id>",Self{data})]
    }
    
}
 impl Handler for CustomHandler{
     fn handle<'a>(&self,req:&'a Request,data:Data)->Outcome<'a>{
         let id= req.get_param::<&RawStr>(0)
             .and_then(|res|res.ok())
             .or_forward(data)?;
         Outcome::from(req,format!("{}-{}",self.data,id))
     }
 }


#[test]
fn test_development_config() {
    config::test_config(rocket::config::Environment::Development);
}

fn rocket() -> rocket::Rocket{
    let always_forward = Route::ranked(1,Get,"/",forward);
    let hello=Route::ranked(2,Get,"/",hi);
    let echo=Route::new(Get,"/echo/<str>",echo_url);
    let name= Route::new(Get,"/<name>",name);
    let post_upload=Route::new(Post,"/",upload);
    let get_upload= Route::new(Get,"/",get_upload);

    let not_found_catcher=Catcher::new(404,not_found_handler);

    rocket::ignite()
        .mount("/",vec![always_forward,hello,echo])
        .mount("/upload",vec![get_upload,post_upload])
        .mount("/hello",vec![name.clone()])
        .mount("/hi",vec![name])
        .mount("/custom",CustomHandler::new("some data"))
        .register(vec![not_found_catcher])

}


fn forward<'a>(_req:&'a Request,data:Data)->Outcome<'a>{
    Outcome::forward(data)

}
fn hi<'a>(req:&'a Request,_:Data)->Outcome<'a>{
    Outcome::from(req,"HELLO")

}

fn echo_url<'a>(req:&'a Request,data:Data)->Outcome<'a>{
    let param= req.get_param::<&RawStr>(1)
    .and_then(|res|res.ok())
    .into_outcome(Status::BadRequest)?;
    Outcome::from(req,RawStr::from_str(param).url_decode())
}
fn name<'a> (req:&'a Request,_:Data)->Outcome<'a>{
    let param = req.get_param::<&'a RawStr>(0)
        .and_then(|res|res.ok())
        .unwrap_or("unnamed".into());
    Outcome::from(req,RawStr::from_str(param).url_decode())
}

fn upload<'a>(req:&'a Request,data:Data)->Outcome<'a>{
    if !req.content_type().map_or(false,|ct|ct.is_plain()) {
        println!(" => content-type of upload must be text/plain ");
        return Outcome::failure(Status::BadRequest);
    }
    let file = File::create(env::temp_dir().join("upload.txt"));
    if let Ok(mut file)=file{
        if let Ok(n)= io::copy(&mut data.open(),&mut file){
            return Outcome::from(req,format!("ok{},bytes uploaded",n));
        }
        println!("=> fail copy");
        Outcome::failure(Status::InternalServerError)
    }else {
        println!("=> cot not open file:{:?}",file.unwrap_err());
        Outcome::failure(Status::InternalServerError)
    }
}
fn get_upload<'a>(req:&'a Request,_:Data)->Outcome<'a>{
    Outcome::from(req,File::open(env::temp_dir().join("unload.txt")).ok())


}
fn not_found_handler<'a>(req:&'a Request)->response::Result<'a>{
    let res=Custom(Status::NotFound,format!("can not find:{}",req.uri()));
    res.respond_to(req)
}

