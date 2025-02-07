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
    ("Leopardi_verses.txt", r#"Alcuni versi di Leopardi:
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
    ("The_Banana.html", "Una banana #Media[banana]"),

    //3
    ("Forbidden_Text.html", "Non scegliere questo testo #Media[do_not_search_this]"),

    //4
    ("Phrases_by_Lillo.txt", r#"Phrases by Lillo:
- a lack of belief in free will is the antidote to hate and judgement
- il disordine è tale finche non viene ordinato
- if you have to ask if you’re a member of a group, you’re probably not."#),

    //5
    ("Mountain_Panoramas.html", r#"One of the best panoramas are next to us,
just walk up on a mountain,
sit in the middle of the forest and watch at the Sparkling snow #Media[sparkling_snow]"#),

    //6
    ("Bigfoot_Sighting.html", r#"Bigfoot Sighting Report
Location: Dense forest near Willow Creek, California
Date and Time: December 12, 2024, 4:45 PM

Image: #Media[big_foot]

Report:
While hiking along an isolated trail, approximately 5 miles from the nearest road, I encountered an unusual figure standing roughly 50 yards away in a clearing.
The figure was enormous, standing between 7 and 8 feet tall, with broad shoulders and a heavily muscled frame.
Its body appeared to be covered in dark, shaggy hair, likely black or very dark brown, and it moved with a distinct upright, bipedal gait."#),

    //7
    ("A_Cat_Life.txt", "A day in the life of a cat: sleep, eat, stare at nothing, and repeat."),

    //8
    ("Famous_Quote.html", "To be or not to be, that is the question. #Media[shakespeare]"),

    //9
    ("Recipe_for_Happiness.txt", r#"Take one sunny day,
Add a sprinkle of laughter,
Mix in some good company,
And serve with warm smiles."#),

    //10
    ("Travel_Dream.html", r#"Imagine waking up to the sound of waves,
A gentle sea breeze,
And a sunrise over turquoise waters. #Media[tropical_paradise]"#),

    //11
    ("Astronomy_Facts.html", "Did you know? The light from the Sun takes about 8 minutes to reach Earth. #Media[sunlight]"),

    //12
    ("Forest_Mysteries.html", r#"The forest is alive with secrets:
Listen closely to the rustling leaves,
And you might hear a hidden story. #Media[forest_story]"#),

    //13
    ("Tech_Innovations.txt", r#"The rise of AI is transforming industries:
From healthcare to space exploration,
The future is already here."#),

    //14
    ("City_Lights.html", "Standing at the rooftop, I watch as the city glows with a million lights. #Media[city_night]"),

    //15
    ("Winter_Tale.txt", r#"The first snow of the year falls gently,
Covering the world in a blanket of white.
A serene and magical moment."#),

    //16
    ("Lost_Artifact.html", r#"A mysterious artifact was discovered in the desert,
Its symbols remain undeciphered to this day. #Media[ancient_artifact]"#),
];


pub const IMAGE_PATHS: [(&str, &str, &str); 10] = [
    ("The_Banana.html", "banana", "path/to/banana.jpeg"),
    ("Forbidden_Text.html", "do_not_search_this", "path/to/do_not_search_this.jpeg"),
    ("Mountain_Panoramas.html", "sparkling_snow", "path/to/sparkling_snow.jpeg"),
    ("Bigfoot_Sighting.html", "big_foot", "path/to/big_foot.jpeg"),
    ("Famous_Quote.html", "shakespeare", "path/to/shakespeare.jpeg"),
    ("Travel_Dream.html", "tropical_paradise", "path/to/tropical_paradise.jpeg"),
    ("Astronomy_Facts.html", "sunlight", "path/to/sunlight.jpeg"),
    ("Forest_Mysteries.html", "forest_story", "path/to/forest_story.jpeg"),
    ("City_Lights.html", "city_night", "path/to/city_night.jpeg"),
    ("Lost_Artifact.html", "ancient_artifact", "path/to/ancient_artifact.jpeg")
];
