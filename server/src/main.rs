use warp::Filter;

fn main() {
    let index = warp::get2()
        .and(warp::path::end())
        .and(warp::fs::file("./dist/index.html"));

    let js = warp::path("main.js")
        .and(warp::path::end())
        .and(warp::fs::file("./dist/main.js"));

    warp::serve(index.or(js))
        .run(([127, 0, 0, 1], 8000));
}
