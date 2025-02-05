use emath::Vec2;
use js_sys::{
    wasm_bindgen::{JsCast, UnwrapThrowExt},
    Date,
};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use web_sys::{Document, HtmlElement, HtmlInputElement};
use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    let date = Date::new_0();
    let (angle1, svg1) = generate_svg(date.clone(), 0);
    let (angle2, svg2) = generate_svg(date.clone(), 1);
    let (angle3, svg3) = generate_svg(date.clone(), 2);

    html! {
        <main>
            // Overlay de la popup
            <div class="overlay" id="overlay"></div>

            // Popup de fin de niveau
            <div class="popup" id="popup">
                <h3 id="the_end"></h3>
                <p id="final_score"></p>
                <p id="final_score_composition"></p>
            </div>
             <table>
                  <tr>
                    <th>{"Guess"}</th>
                    <th>{"The"}</th>
                    <th>{"Angle"}</th>
                  </tr>
                  <tr>
                    <td>{svg1}</td>
                    <td>{svg2}</td>
                    <td>{svg3}</td>
                  </tr>
                  <tr>
                    <td><input type="number" id="guess1" min="0" max="360" onkeydown={move |e| check_input(e, angle1, "guess1")}/></td>
                    <td><input type="number" id="guess2" min="0" max="360" hidden=true onkeydown={move |e| check_input(e, angle2, "guess2")}/></td>
                    <td><input type="number" id="guess3" min="0" max="360" hidden=true onkeydown={move |e| check_input(e, angle3, "guess3")}/></td>
                  </tr>
                  <tr>
                    <td hidden=true id="answer1"></td>
                    <td hidden=true id="answer2"></td>
                    <td hidden=true id="answer3"></td>
                  </tr>
            </table>
        </main>
    }
}

fn check_input(e: KeyboardEvent, angle: usize, id: &'static str) {
    if e.key() == "Enter" {
        let document = web_sys::window().unwrap_throw().document().unwrap_throw();
        let input: HtmlInputElement = get_element_by_id(&document, id);

        input.set_read_only(true);
        let guessed_angle: usize = input.value().parse().unwrap_throw();

        let (next, answer, score_id) = match id {
            "guess1" => (Some("guess2"), "answer1", "score1"),
            "guess2" => (Some("guess3"), "answer2", "score2"),
            "guess3" => (None, "answer3", "score3"),
            _ => return, // should never happen
        };

        let input: HtmlElement = get_element_by_id(&document, answer);
        let error = guessed_angle.abs_diff(angle) as f64;
        let error_percentage = (360.0 - error) / 360.0 * 100.0;
        let class = match error_percentage {
            // Must be the first branch otherwise it collides with `good`
            100.0 => "perfect",
            ..=33.33 => "failure",
            33.33..=66.66 => "error",
            66.66..=90.00 => "almost",
            95.00.. => "good",
            // everything else is a failure
            _ => "failure",
        };
        let points = error_percentage / 3.0;
        input.set_inner_html(&format!(
            r#"<span>{angle}</span><br/><span id="{score_id}" class="{class}"/>+{points:.2}</span>"#
        ));
        input.set_hidden(false);
        if let Some(next) = next {
            let next_input: HtmlInputElement = get_element_by_id(&document, next);
            next_input.set_hidden(false);
            next_input.focus().unwrap_throw();
        } else {
            finish_game(&document);
        }
    }
}

fn finish_game(document: &Document) {
    let score1: HtmlElement = get_element_by_id(document, "score1");
    let score2: HtmlElement = get_element_by_id(document, "score2");
    let score3: HtmlElement = get_element_by_id(document, "score3");
    let score1: f32 = score1.text_content().unwrap_throw().parse().unwrap();
    let score2: f32 = score2.text_content().unwrap_throw().parse().unwrap();
    let score3: f32 = score3.text_content().unwrap_throw().parse().unwrap();

    let title: HtmlElement = get_element_by_id(document, "the_end");
    let final_score: HtmlElement = get_element_by_id(document, "final_score");
    let composition: HtmlElement = get_element_by_id(document, "final_score_composition");

    title.set_text_content(Some(generate_title_for_score(score1 + score2 + score3)));
    final_score.set_text_content(Some(&format!("Score: {}", score1 + score2 + score3)));
    composition.set_text_content(Some(&format!("{} + {} + {}", score1, score2, score3)));

    let overlay: HtmlElement = get_element_by_id(document, "overlay");
    let popup: HtmlElement = get_element_by_id(document, "popup");
    overlay.style().set_property("display", "block").unwrap();
    popup.style().set_property("display", "block").unwrap();
}

fn get_element_by_id<T: JsCast>(document: &Document, id: &str) -> T {
    document
        .get_element_by_id(id)
        .unwrap_throw()
        .dyn_into()
        .unwrap_throw()
}

fn generate_svg(date: Date, part: u32) -> (usize, Html) {
    let seed = date.get_full_year() * date.get_month() * date.get_date() + part;
    let mut rng = SmallRng::seed_from_u64(seed as u64);
    let angle = (rng.random_range(0..360) as f32).to_radians();
    // We're going to draw both lines in a circle with a diameter of 100.
    // - It's center will be at 50,50.
    // - We want to offset the first line by a random angle from the horizontal line.
    // - The second one will be built from here.
    let offset_by = (rng.random_range(0..360) as f32).to_radians();
    let rayon = 50.0;

    let center = Vec2::splat(rayon);
    let first = Vec2 {
        x: offset_by.cos(),
        y: offset_by.sin(),
    } * rayon
        + center;

    let second = Vec2 {
        x: (offset_by + angle).cos(),
        y: (offset_by + angle).sin(),
    } * rayon
        + center;

    let angle = angle.to_degrees() as usize;
    let diam = (rayon * 2.0).to_string();
    let mask = format!(
        "{},{} {},{} {},{} {},{}",
        rayon, rayon, first.x, first.y, second.x, second.y, rayon, rayon,
    );
    let svg = html! {
        <svg
          version="1.1"
          baseProfile="full"
          width={diam.to_string()}
          height={diam.to_string()}
          xmlns="http://www.w3.org/2000/svg">

          <defs>
            <@{"clipPath"} id={format!("keepAngle-{part}")}>
             <polygon points={mask.to_string()} />
            </@>
          </defs>

          <circle cx={rayon.to_string()} cy={rayon.to_string()} r={rayon.to_string()}/>
          if angle < 180 {
              <circle cx={rayon.to_string()} cy={rayon.to_string()} r={(rayon / 2.0).to_string()} clip-path={format!("url(#keepAngle-{part})")} fill="red" />
              <circle cx={rayon.to_string()} cy={rayon.to_string()} r={(rayon / 2.0 - 1.0).to_string()} fill="black" />
          } else {
              <circle cx={rayon.to_string()} cy={rayon.to_string()} r={(rayon / 2.0).to_string()} fill="red" />
              <circle cx={rayon.to_string()} cy={rayon.to_string()} r={(rayon / 2.0 - 1.0).to_string()} fill="black" />
              <polygon points={mask.to_string()} fill="black"/>
          }
          <line
                x1={center.x.to_string()} y1={center.y.to_string()}
                x2={first.x.to_string()} y2={first.y.to_string()} stroke="white"
            />
          <line
                x1={center.x.to_string()} y1={center.y.to_string()}
                x2={second.x.to_string()} y2={second.y.to_string()} stroke="white"
            />
        </svg>
    };
    (angle, svg)
}

fn generate_title_for_score(score: f32) -> &'static str {
    match score {
        100.0 => "Wow, cheating in a game like that? Really?",
        00.00..=10.00 => "Do you have a humiliation kink?",
        10.00..=20.00 => "You don't have anything better to do?",
        20.00..=30.00 => "Nice, your score matches your IQ",
        30.00..=40.00 => {
            "If you are looking for information on the Germanic invaders, you're not on the right website"
        }
        40.00..=50.00 => "Just forget this website I don't want to see your face tomorrow",
        50.00..=60.00 => "My dog plays better than you",
        60.00..=70.00 => "Did you understand the purpose of this game?",
        70.00..=80.00 => "At this point, picking random numbers may yield better results",
        80.00..=90.00 => "You're supposed to think before typing",
        90.00..=100.00 => "Not bad for a blind person",
        _ => "I lost your score but it was probably bad anyway",
    }
}
