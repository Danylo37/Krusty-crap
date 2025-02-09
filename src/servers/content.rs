use std::collections::HashMap;
use rand::random;

const N_FILES: usize = 16;

pub fn choose_random_texts() -> Vec<(String, String)> {
    let trying_closures = |x: u8| {
        if x < 4 {
            return x + 3;
        } else {
            x
        }
    };

    let n_files = trying_closures(random::<u8>() % (N_FILES as u8));

    let mut vec_files: Vec<(String, String)> = Vec::new();
    if random::<u8>() % 2 == 0 {
        for i in 0..n_files {
            // Access the first element of the tuple (index 0)
            vec_files.push((TEXT[i as usize].0.to_string(), TEXT[i as usize].1.to_string()));
        }
    } else {
        for i in (0..n_files).rev() {
            // Access the first element of the tuple (index 0)
            vec_files.push((TEXT[i as usize].0.to_string(), TEXT[i as usize].1.to_string()));
        }
    }
    vec_files
}

pub fn get_media(vec_files: Vec<(String, String)>) -> HashMap<String, String> {
    IMAGE_PATHS.iter().filter_map(|(title, ref_s, media)| {
        // Check if the `title` exists in `vec_files`
        if vec_files.iter().any(|(chosen_title, _)| chosen_title == title) {
            // Return the tuple (ref_s, media) if the condition is true
            Some((ref_s.to_string(), media.to_string()))
        } else {
            // Discard the element by returning None
            None
        }
    }).collect::<HashMap<String, String>>() // Collect into HashMap
}


pub const TEXT: [(&str, &str); N_FILES] = [
    //1
    ("leopardi_verses.txt", r#"Alcuni versi di Leopardi:
Ma perchè dare al sole,
Perchè reggere in vita
Chi poi di quella consolar convenga?
Se la vita è sventura,
Perchè da noi si dura?
Intatta luna, tale
E’ lo stato mortale.
Ma tu mortal non sei,
E forse del mio dir poco ti cale."#),

    //2
    ("the_banana.html", "Una banana #Media[banana]"),

    //3
    ("forbidden_text.html", "Non scegliere questo testo #Media[do_not_search_this]"),

    //4
    ("phrases_by_lillo.txt", r#"Phrases by Lillo:
- a lack of belief in free will is the antidote to hate and judgement
- il disordine è tale finche non viene ordinato
- if you have to ask if you’re a member of a group, you’re probably not."#),

    //5
    ("mountain_panoramas.html", r#"One of the best panoramas are next to us,
just walk up on a mountain,
sit in the middle of the forest and watch at the Sparkling snow #Media[sparkling_snow]"#),

    //6
    ("bigfoot_sighting.html", r#"Bigfoot Sighting Report
Location: Dense forest near Willow Creek, California
Date and Time: December 12, 2024, 4:45 PM

Image: #Media[big_foot]

Report:
While hiking along an isolated trail, approximately 5 miles from the nearest road, I encountered an unusual figure standing roughly 50 yards away in a clearing.
The figure was enormous, standing between 7 and 8 feet tall, with broad shoulders and a heavily muscled frame.
Its body appeared to be covered in dark, shaggy hair, likely black or very dark brown, and it moved with a distinct upright, bipedal gait."#),

    //7
    ("a_cat_life.txt", "A day in the life of a cat: sleep, eat, stare at nothing, and repeat."),

    //8
    ("famous_quote.html", "To be or not to be, that is the question. #Media[shakespeare]"),

    //9
    ("recipe_for_happiness.txt", r#"Take one sunny day,
Add a sprinkle of laughter,
Mix in some good company,
And serve with warm smiles."#),

    //10
    ("travel_dream.html", r#"Imagine waking up to the sound of waves,
A gentle sea breeze,
And a sunrise over turquoise waters. #Media[tropical_paradise]"#),

    //11
    ("astronomy_facts.html", "Did you know? The light from the Sun takes about 8 minutes to reach Earth. #Media[sunlight]"),

    //12
    ("forest_mysteries.html", r#"The forest is alive with secrets:
Listen closely to the rustling leaves,
And you might hear a hidden story. #Media[forest_story]"#),

    //13
    ("tech_innovations.txt", r#"The rise of AI is transforming industries:
From healthcare to space exploration,
The future is already here."#),

    //14
    ("city_lights.html", "Standing at the rooftop, I watch as the city glows with a million lights. #Media[city_night]"),

    //15
    ("winter_tale.txt", r#"The first snow of the year falls gently,
Covering the world in a blanket of white.
A serene and magical moment."#),

    //16
    ("lost_artifact.html", r#"A mysterious artifact was discovered in the desert,
Its symbols remain undeciphered to this day. #Media[ancient_artifact]"#),
];


pub const IMAGE_PATHS: [(&str, &str, &str); 15] = [
    ("the_banana.html", "banana", "\static\content_objects\content_images\banana.png"),
    ("forbidden_text.html", "do_not_search_this", "\static\content_objects\content_images\do_not_search_this.jpg"),
    ("mountain_panoramas.html", "sparkling_snow", "\static\content_objects\content_images\do_not_search_this.jpg"),
    ("bigfoot_sighting.html", "big_foot", "\static\content_objects\content_images\big_foot.png"),
    ("famous_quote.html", "shakespeare", "\static\content_objects\content_images\shakespeare.png"),
    ("travel_dream.html", "tropical_paradise", "\static\content_objects\content_images\tropical_paradise.jpg"),
    ("astronomy_facts.html", "sunlight", "\static\content_objects\content_images\sunlight.jpg"),
    ("forest_mysteries.html", "forest_story", "\static\content_objects\content_images\forest_story.jpg"),
    ("city_lights.html", "city_night", "\static\content_objects\content_images\city_night.jpg"),
    ("lost_artifact.html", "ancient_artifact", "\static\content_objects\content_images\ancient_artifact.jpg"),
    // New content -----------------------
    ("forgotten_poem.html", "library_wonder", "\static\content_objects\content_images\library_wonder.jpg"),
    ("old_map.html", "hidden_cave", "\static\content_objects\content_images\hidden_cave.jpg"),
    ("desert_whispers.html", "desert_oasis", "\static\content_objects\content_images\desert_oasis.jpg"),
    ("cyber_rebellion.html", "futuristic_city", "\static\content_objects\content_images\futuristic_city.jpg"),
    ("abandoned_castle.jpeg", "abandoned_castle","\static\content_objects\content_images\abandoned_castle.jpg"),
];
