use std::collections::HashMap;
use rand::{random, thread_rng};
use rand::seq::SliceRandom;

const N_FILES: usize = 25;

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
    let mut randomized_indexes: Vec<usize> = (0..TEXT.len()).collect();
    randomized_indexes.shuffle(&mut thread_rng());

    if random::<u8>() % 2 == 0 {
        for &i in randomized_indexes.iter().take(n_files as usize) {
            vec_files.push((TEXT[i].0.to_string(), TEXT[i].1.to_string()));
        }

    } else {
        for &i in randomized_indexes.iter().take(n_files as usize).rev(){
            vec_files.push((TEXT[i].0.to_string(), TEXT[i].1.to_string()));
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
    ("the_banana.html", r#"<body style="font-family:Arial, sans-serif; line-height:1.6; max-width:800px; margin:auto; padding:20px; background-color:#f7f7f7; color:#333;">

    <h1 style="margin-top: 0; text-align:center; font-size:2.5em; color:#f2b600; letter-spacing: 2px;">The Banana</h1>

    <p style="margin-bottom: 15px; font-size: 1.1em; background-color: rgba(255, 255, 255, 0.8); padding: 10px 15px; border-radius: 8px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);">
        The banana is one of the most widely consumed fruits in the world. It is known for its sweet taste, soft texture, and nutritional benefits.
    </p>

    <div style="text-align:center; margin:20px 0;">
        #Media[banana]
    </div>

    <p style="margin-bottom: 15px; font-size: 1.1em; background-color: rgba(255, 255, 255, 0.8); padding: 10px 15px; border-radius: 8px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);">
        Originating from Southeast Asia, bananas are now grown in many tropical and subtropical regions. They are a rich source of potassium, vitamins, and fiber, making them an essential part of a healthy diet.
    </p>

    <p style="margin-bottom: 15px; font-size: 1.1em; background-color: rgba(255, 255, 255, 0.8); padding: 10px 15px; border-radius: 8px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);">
        Bananas are often eaten fresh, but they can also be used in smoothies, baked goods, and even savory dishes. Their natural sweetness makes them a great alternative to refined sugar in recipes.
    </p>

    <p style="margin-bottom: 15px; font-size: 1.1em; background-color: rgba(255, 255, 255, 0.8); padding: 10px 15px; border-radius: 8px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);">
        Beyond their culinary uses, bananas have cultural significance in many societies. In some places, the banana leaf is used as a natural plate, while in others, the fruit plays a role in religious ceremonies and traditions.
    </p>

    <p style="margin-bottom: 15px; font-size: 1.1em; background-color: rgba(255, 255, 255, 0.8); padding: 10px 15px; border-radius: 8px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);">
        Whether eaten alone, blended into a smoothie, or baked into a cake, the banana remains a favorite fruit worldwide.
    </p>

</body>
"#),

    //3
    ("forbidden_text.html", r#"<body style="font-family:Arial, sans-serif; line-height:1.6; max-width:800px; margin:auto; padding:20px; background-color:#f8f8f8; color:#333;">

    <h1 style="margin-top: 0; text-align:center; color:#ff3333; font-size:2.5em; letter-spacing: 2px; text-transform: uppercase;">⚠ Forbidden Text ⚠</h1>

    <p style="margin-bottom: 15px; font-size: 1.1em; background-color: rgba(0, 0, 0, 0.1); padding: 10px 15px; border-radius: 8px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1); line-height: 2rem;">
        There exist certain texts that have been hidden from the public eye for generations. Some say they contain knowledge too dangerous to be shared. Others believe they are mere superstition.
    </p>

    <div style="text-align:center; margin:20px 0;">
        #Media[do_not_search_this]
    </div>

    <p style="margin-bottom: 15px; font-size: 1.1em; background-color: rgba(0, 0, 0, 0.1); padding: 10px 15px; border-radius: 8px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1); line-height: 2rem;">
        The origins of this mysterious writing remain unknown. Fragments have surfaced in ancient libraries, secret societies, and encrypted manuscripts, yet no one has ever deciphered the full meaning.
    </p>

    <p style="margin-bottom: 15px; font-size: 1.1em; background-color: rgba(0, 0, 0, 0.1); padding: 10px 15px; border-radius: 8px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1); line-height: 2rem;">
        Many who have attempted to read the forbidden text speak of strange occurrences—whispers in the dark, flickering lights, and an overwhelming sense of unease. Could it all be coincidence, or is there something truly hidden within these words?
    </p>

    <p style="margin-bottom: 15px; font-size: 1.1em; background-color: rgba(0, 0, 0, 0.1); padding: 10px 15px; border-radius: 8px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1); line-height: 2rem;">
        Some scholars warn that certain knowledge is best left undiscovered. But for those who are curious, the question remains: will you dare to seek the truth, or will you turn back before it's too late?
    </p>

</body>
"#),

    //4
    ("phrases_by_lillo.txt", r#"Phrases by Lillo:
- a lack of belief in free will is the antidote to hate and judgement
- il disordine è tale finche non viene ordinato
- if you have to ask if you’re a member of a group, you’re probably not."#),

    //5
    ("mountain_panoramas.html", r#"<body style="font-family:Arial, sans-serif; line-height:1.6; max-width:800px; margin:auto; padding:20px; background-color:#eef2f3; color:#333;">

    <h1 style="margin-top: 0; text-align:center; color:#1e4f3d; font-size:2.5em; letter-spacing: 1.5px; text-transform: uppercase; padding-bottom: 10px; border-bottom: 2px solid #1e4f3d;">
        Breathtaking Mountain Panoramas
    </h1>

    <p style="margin-bottom: 15px; font-size: 1.1em; background-color: rgba(0, 0, 0, 0.05); padding: 10px 15px; border-radius: 8px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1); line-height: 2rem;">
        Some of the most stunning views on Earth can be found in the mountains. Whether covered in lush greenery or blanketed in fresh snow, these landscapes offer a sense of peace and wonder.
    </p>

    <p style="margin-bottom: 15px; font-size: 1.1em; background-color: rgba(0, 0, 0, 0.05); padding: 10px 15px; border-radius: 8px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1); line-height: 2rem;">
        Walking along a mountain trail, the crisp air fills your lungs. The silence is only broken by the rustling of leaves and the distant call of a bird. Each step brings you closer to a perfect view, one that few get to witness.
    </p>

    <div style="text-align:center; margin:20px 0;">
        #Media[sparkling_snow]
    </div>

    <p style="margin-bottom: 15px; font-size: 1.1em; background-color: rgba(0, 0, 0, 0.05); padding: 10px 15px; border-radius: 8px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1); line-height: 2rem;">
        Sitting in the middle of a forest clearing, you gaze at the untouched snow glistening under the sun. The world feels still, frozen in time, as nature displays its beauty in the simplest of ways.
    </p>

    <p style="margin-bottom: 15px; font-size: 1.1em; background-color: rgba(0, 0, 0, 0.05); padding: 10px 15px; border-radius: 8px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1); line-height: 2rem;">
        Whether you seek adventure or solitude, the mountains have something for everyone. They remind us that the best panoramas are often right beside us, waiting to be discovered.
    </p>

</body>
"#),

    //6
    ("bigfoot_sighting.html", r#"<body style="font-family:Arial, sans-serif; line-height:1.6; max-width:800px; margin:auto; padding:20px; background-color:#f2f0e6; color:#4e3629;">

    <h1 style="margin-top: 0; text-align:center; color:#6a4e3c; font-size:2.5em; letter-spacing: 1.5px; text-transform: uppercase; padding-bottom: 10px; border-bottom: 2px solid #6a4e3c;">
        Bigfoot Sighting Report
    </h1>

    <p style="font-weight:bold; margin-bottom: 10px; line-height: 2rem;">
        <strong>Location:</strong> Dense forest near Willow Creek, California
    </p>
    <p style="font-weight:bold; margin-bottom: 10px; line-height: 2rem;">
        <strong>Date and Time:</strong> December 12, 2024, 4:45 PM
    </p>

    <div style="text-align:center; margin:20px 0;">
        #Media[big_foot]
    </div>

    <h2 style="font-size:1.8em; color:#6a4e3c; margin-top: 30px; text-align:center; border-bottom: 1px solid #6a4e3c; padding-bottom: 10px;">
        Witness Report
    </h2>

    <p style="margin-bottom: 20px; background-color: rgba(0, 0, 0, 0.05); padding: 15px; border-radius: 8px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1); font-size:1.1em; line-height: 2rem;">
        While hiking along an isolated trail, approximately 5 miles from the nearest road, I encountered an unusual figure standing roughly 50 yards away in a clearing.
    </p>

    <p style="margin-bottom: 20px; background-color: rgba(0, 0, 0, 0.05); padding: 15px; border-radius: 8px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1); font-size:1.1em; line-height: 2rem;">
        The figure was enormous, standing between 7 and 8 feet tall, with broad shoulders and a heavily muscled frame. Its body appeared to be covered in dark, shaggy hair, likely black or very dark brown, and it moved with a distinct upright, bipedal gait.
    </p>

    <p style="margin-bottom: 20px; background-color: rgba(0, 0, 0, 0.05); padding: 15px; border-radius: 8px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1); font-size:1.1em; line-height: 2rem;">
        For a few moments, we stood frozen—me, trying to process what I was seeing, and the creature, seemingly studying me in return. Then, with a slow yet deliberate movement, it turned and disappeared into the dense undergrowth, leaving behind only a series of deep footprints in the soft earth.
    </p>

    <p style="margin-bottom: 20px; background-color: rgba(0, 0, 0, 0.05); padding: 15px; border-radius: 8px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1); font-size:1.1em; line-height: 2rem;">
        Could it have been a hoax? Perhaps. But the eerie silence that followed, the unsettling weight of the moment—it felt real. Too real.
    </p>

</body>
"#),

    //7
    ("a_cat_life.txt", "A day in the life of a cat is a carefully balanced cycle of elegance, mystery, and absolute laziness.

Wake up. Stretch dramatically. Yawn as if the weight of the world rests upon your tiny shoulders. Walk to the food bowl with the grace of royalty, only to stare at it in disappointment when it is not filled to your liking. Meow loudly. Human obeys. Eat exactly three bites before walking away.

Find a sunny spot. Curl up. Sleep. Dream of chasing things you have no intention of catching. Wake up suddenly as if you remembered something urgent, only to realize it was nothing. Stare at the wall for an uncomfortably long time.

At 3 AM, sprint across the house as if possessed by an unseen force. Knock something off a table for reasons only you understand. Watch your human sigh in defeat. Mission accomplished.

Sleep again. Repeat the next day. Such is the life of a cat—simple, yet undeniably perfect."),

    //8
    ("famous_quote.html", r#"
    <body style="font-family:Arial, sans-serif; line-height:1.6; max-width:800px; margin:auto; padding:20px; background-color:#f0f0f0; color:#333;">

        <h1 style="margin-top: 0; text-align:center; font-size:2.5em; color:#2d2d2d; font-weight:700; letter-spacing: 1.5px; text-transform: uppercase; padding-bottom: 10px; border-bottom: 2px solid #2d2d2d;">
            To Be or Not to Be
        </h1>

        <p style="font-size:1.2em; margin-bottom: 20px; line-height: 2rem;">
            "To be, or not to be, that is the question." This famous line from Shakespeare’s *Hamlet* has resonated through the ages, posing one of life’s most profound questions. It speaks to the internal struggle, the contemplation of existence itself, and the choice between enduring suffering or ending it.
        </p>

        <div style="text-align:center; margin:20px 0;">
            #Media[shakespeare]
        </div>

        <p style="font-size:1.1em; background-color: rgba(0, 0, 0, 0.05); padding: 15px; border-radius: 8px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1); margin-bottom: 20px; line-height: 2rem;">
            Shakespeare’s *Hamlet* is one of the most well-known works of literature, and this quote encapsulates the torment of the play's protagonist. As Hamlet debates the merit of life and death, his words echo through time, exploring the universal theme of human existence and its meaning.
        </p>

        <p style="font-size:1.1em; background-color: rgba(0, 0, 0, 0.05); padding: 15px; border-radius: 8px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1); margin-bottom: 20px; line-height: 2rem;">
            Even today, over 400 years after it was written, the question remains relevant. We all face moments of doubt, moments when we question our purpose, our place in the world, and the choices we must make.
        </p>

        <p style="font-size:1.1em; background-color: rgba(0, 0, 0, 0.05); padding: 15px; border-radius: 8px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1); margin-bottom: 20px; line-height: 2rem;">
            Perhaps the real question is not whether to live or die, but how we live—how we embrace the moments of uncertainty and continue to move forward despite them.
        </p>

    </body>
    "#),

    //9
    ("recipe_for_happiness.txt", r#"Recipe for Happiness

Ingredients:
- 1 bright and sunny day (or a cozy rainy one, if preferred)
- A generous sprinkle of laughter
- A handful of good company
- 2 cups of kindness
- A dash of curiosity
- A pinch of adventure
- Unlimited warm smiles

Instructions:
1. Start by embracing the day with gratitude. Whether the sun is shining or the rain is tapping gently against the window, find joy in the moment.
2. Add a sprinkle of laughter—this is the secret ingredient. Let it bubble up naturally, whether from a joke, a shared memory, or the simple absurdity of life.
3. Mix in good company. Friends, family, or even a kind stranger—happiness grows best when shared.
4. Gently fold in kindness. A small gesture, a thoughtful word, or an act of generosity can enhance the flavor of joy.
5. Sprinkle in a dash of curiosity—try something new, explore a new place, or simply ask "why" more often.
6. If you're feeling adventurous, add a pinch of spontaneity. A last-minute plan, an unexpected detour, or dancing when no one is watching can make life richer.
7. Serve immediately, with warm smiles and open hearts.

Tip: Best enjoyed daily. Pairs well with deep conversations, stargazing, and quiet moments of reflection.
"#),

    //10
    ("travel_dream.html", r#"
    <body style="font-family:Arial, sans-serif; line-height:1.6; max-width:800px; margin:auto; padding:20px; background-color:#f0f8ff; color:#333;">

        <h1 style="margin-top: 0; text-align:center; font-size:2.5em; color:#2a8c8d; font-weight:700; letter-spacing: 1.5px; text-transform: uppercase; padding-bottom: 10px; border-bottom: 2px solid #2a8c8d;">
            A Dream of Travel
        </h1>

        <p style="font-size:1.2em; margin-bottom: 20px; text-align:justify; line-height: 2rem;">
            Imagine waking up to the sound of waves gently crashing against the shore, each wave a soft lullaby from the ocean. As the morning sun rises, a warm breeze carries the scent of saltwater and tropical flowers, filling your lungs with fresh air.
        </p>

        <div style="text-align:center; margin:20px 0;">
            #Media[tropical_paradise]
        </div>

        <p style="font-size:1.2em; margin-bottom: 20px; text-align:justify; line-height: 2rem;">
            The sky is painted in shades of pink and orange as the sun rises over turquoise waters. The horizon stretches out endlessly, offering a view so serene that it almost feels like a dream itself. You can hear the rustling of palm trees swaying in the breeze, their leaves whispering secrets of the tropical paradise they call home.
        </p>

        <p style="font-size:1.2em; margin-bottom: 20px; text-align:justify; line-height: 2rem;">
            For a moment, everything feels still. You sit back, close your eyes, and just breathe in the beauty of the world around you. This is a place where time slows down, where worries disappear, and all that remains is peace and wonder.
        </p>

        <p style="font-size:1.2em; margin-bottom: 20px; text-align:justify; line-height: 2rem;">
            This dream is not far from reality; it exists in the hearts of those who long to explore, to escape, and to find solace in the arms of nature. And perhaps, one day, you’ll find yourself there—waking up to the sound of waves and the promise of a new adventure.
        </p>

    </body>
    "#),

    //11
    ("astronomy_facts.html", r#"
    <body style="font-family:Arial, sans-serif; line-height:1.6; max-width:800px; margin:auto; padding:20px; background-color:#f0f8ff; color:#333;">

        <h1 style="margin-top: 0; text-align:center; color:#2a8c8d;">
            Fascinating Astronomy Facts
        </h1>

        <p style="font-size:1.2em; margin-bottom: 20px; text-align:justify; line-height: 2rem;">
            The universe is vast and full of wonders. From the planets that orbit distant stars to the far reaches of black holes, our understanding of space continues to grow. Here are some mind-blowing astronomy facts that will leave you in awe.
        </p>

        <p style="font-size:1.2em; margin-bottom: 20px; text-align:justify; line-height: 2rem;">
            <strong>1. A day on Venus is longer than a year on Venus.</strong> Venus has an extremely slow rotation, taking about 243 Earth days to complete one rotation on its axis. Meanwhile, it only takes 225 Earth days to orbit the Sun. This makes a day on Venus longer than its year.
        </p>

        <p style="font-size:1.2em; margin-bottom: 20px; text-align:justify; line-height: 2rem;">
            <strong>2. There are more stars in the universe than grains of sand on Earth.</strong> It is estimated that there are around 100 billion galaxies in the observable universe, each containing billions or even trillions of stars. The total number of stars is far greater than the number of grains of sand on all of Earth's beaches.
        </p>

        <p style="font-size:1.2em; margin-bottom: 20px; text-align:justify; line-height: 2rem;">
            <strong>3. The largest known star is UY Scuti.</strong> UY Scuti, located in the constellation Scutum, is considered one of the largest known stars. Its diameter is roughly 1,700 times that of the Sun, making it an absolute giant in the cosmos.
        </p>

        <p style="font-size:1.2em; margin-bottom: 20px; text-align:justify; line-height: 2rem;">
            <strong>4. A teaspoon of a neutron star would weigh about 6 billion tons.</strong> Neutron stars are incredibly dense remnants of supernova explosions. The material inside them is so dense that a single teaspoon would have a mass of about 6 billion tons!
        </p>

        <div style="text-align:center; margin:20px 0;">
            #Media[astronomy_image]
        </div>

        <p style="font-size:1.2em; margin-bottom: 20px; text-align:justify; line-height: 2rem;">
            The universe is filled with mysteries, and we have only scratched the surface. Who knows what future discoveries will reveal? Keep looking up!
        </p>

    </body>
    "#),

    //12
    ("forest_mysteries.html", r#"
    <body style="font-family:Arial, sans-serif; line-height:1.6; max-width:800px; margin:auto; padding:20px; background-color:#f4f7f2; color:#2c3e3a;">

        <h1 style="margin-top: 0; text-align:center; color:#3e6e5f;">
            Forest Mysteries
        </h1>

        <p style="line-height: 2rem; text-align: justify;">
            The forest is alive with secrets, each tree and rustling leaf whispering a story from ancient times. Walk beneath the emerald canopy, and you may feel the weight of history pressing against your shoulders, as if unseen eyes are watching from the shadows.
        </p>

        <div style="text-align:center; margin:20px 0;">
            #Media[forest_story]
        </div>

        <p style="line-height: 2rem; text-align: justify;">
            Listen closely to the wind weaving through the branches—it carries voices of those who walked these paths long before you. A broken twig underfoot, the distant hoot of an owl, the sudden hush that falls over the trees—all could be signs of something hidden just beyond your sight.
        </p>

        <p style="line-height: 2rem; text-align: justify;">
            Some say that if you venture deep enough, you may stumble upon a place untouched by time. A forgotten shrine covered in ivy, an ancient tree older than any map, or perhaps a clearing where the world feels just a little different—as if reality itself bends to the will of the forest.
        </p>

        <p style="line-height: 2rem; text-align: justify;">
            But beware. Not all who seek the forest’s secrets return with answers. Some stories are meant to remain hidden, buried beneath roots and shadows, waiting for those who dare to listen.
        </p>

    </body>
    "#),

    //13
    ("tech_innovations.txt", r#"Tech Innovations: A New Era Unfolds

The rise of AI is transforming industries at an unprecedented pace.
What was once the realm of science fiction is now reality, shaping the way we live, work, and explore the universe.

Key areas of impact:

- **Healthcare:** AI-driven diagnostics can detect diseases earlier than ever before, robotic surgeries are improving precision, and personalized medicine is tailoring treatments to individual patients.
- **Space Exploration:** Autonomous rovers, AI-assisted navigation, and machine learning models are helping us explore distant planets, analyze cosmic data, and even search for extraterrestrial life.
- **Automation & Industry:** Smart factories optimize production, reducing waste and increasing efficiency, while AI-powered logistics ensure supply chains run smoothly across the globe.
- **Everyday Life:** From virtual assistants and recommendation algorithms to self-driving cars, AI is seamlessly integrating into our daily routines.
- **Cybersecurity:** Advanced AI models detect and neutralize cyber threats in real time, keeping digital infrastructures secure.
- **Creativity & Art:** AI-generated music, paintings, and literature challenge traditional notions of human creativity, blurring the line between artist and algorithm.

The future is already here, and it is learning, evolving, and redefining what’s possible. The only question left is: how will we shape this new technological frontier?
"#),

    //14
    ("city_lights.html", r#"
    <body style="font-family:Arial, sans-serif; line-height:1.6; max-width:800px; margin:auto; padding:20px; background-color:#e9e9e9; color:#333;">

        <h1 style="text-align:center; color:#4c4f5c;">
            City Lights
        </h1>

        <p style="line-height: 2rem; text-align: justify;">
            From the rooftop, the city stretches before me, a vast sea of lights flickering like distant stars. Each window, each neon sign, each glowing streetlamp tells a silent story—a heartbeat in the metropolis that never truly sleeps.
        </p>

        <div style="text-align:center; margin:20px 0;">
            #Media[city_night]
        </div>

        <p style="line-height: 2rem; text-align: justify;">
            The skyline stands tall, jagged and luminous, cutting through the darkness like a modern constellation. Below, the streets hum with the quiet symphony of life—the distant murmur of conversations, the rhythmic pulse of car horns, the occasional burst of laughter from a late-night café.
        </p>

        <p style="line-height: 2rem; text-align: justify;">
            There’s something surreal about this moment—standing high above the world, watching the city move like an intricate clockwork. Each light below represents a story, a dream, a fleeting moment in time, all woven into the fabric of the night.
        </p>

        <p style="line-height: 2rem; text-align: justify;">
            And as I take it all in, I wonder—who else stands at their own window, gazing into the same night, lost in thought? For all its chaos, the city has a way of making strangers feel connected, even in solitude.
        </p>

    </body>
    "#),

    //15
    ("winter_tale.txt", r#"Winter Tale

The first snow of the year falls gently,
Covering the world in a blanket of white.
A serene and magical moment, where time seems to slow,
And the air is filled with quiet wonder.

Footsteps vanish as quickly as they appear,
Soft flakes settling on rooftops and tree branches,
Transforming the ordinary into something almost otherworldly.

Children laugh in the distance, building snowmen,
While the glow of streetlights catches the falling snow,
Turning each flake into a tiny, fleeting star.

The world feels peaceful,
As if winter itself is whispering a lullaby,
Reminding us to pause, to breathe,
And to embrace the beauty of the cold.
"#),

    //16
    ("lost_artifact.html", r#"
    <body style="font-family:Arial, sans-serif; line-height:1.6; max-width:800px; margin:auto; padding:20px; background-color:#fafafa; color:#333;">

        <h1 style="text-align:center; color:#4d4b39;">
            The Lost Artifact
        </h1>

        <p style="line-height: 2rem; text-align: justify;">
            Deep within the endless expanse of the desert, buried beneath shifting sands, a mysterious artifact has been unearthed. Its origins remain unknown, and the symbols etched onto its surface continue to defy all attempts at deciphering. Some believe it holds the key to an ancient civilization, long forgotten by time.
        </p>

        <div style="text-align:center; margin:20px 0;">
            #Media[ancient_artifact]
        </div>

        <p style="line-height: 2rem; text-align: justify;">
            Crafted from a metal unlike any known to modern science, the artifact gleams under the relentless desert sun, its surface untouched by time. Archaeologists and historians alike are baffled—some theorize it belonged to a lost kingdom, while others whisper of something more otherworldly.
        </p>

        <p style="line-height: 2rem; text-align: justify;">
            Despite years of study, the cryptic markings that adorn its structure refuse to yield their secrets. No known language matches its script, leaving researchers with far more questions than answers. Is it a relic from a forgotten people, or evidence of something beyond our understanding?
        </p>

        <p style="line-height: 2rem; text-align: justify;">
            Until its secrets are unveiled, the artifact remains a symbol of humanity’s unending curiosity—a reminder that, even in the modern age, the world still holds mysteries waiting to be discovered.
        </p>

    </body>
    "#),

    //17
    ("hidden_wisdom.txt", "The greatest secrets are not kept, but overlooked.
In the hustle of daily life, we often miss the things that truly matter—hidden in plain sight, waiting to be discovered. The wisdom we seek is often not hidden in distant lands or ancient texts, but right before our eyes, in the small moments, the quiet whispers of nature, or the simple truths that we forget to acknowledge.

Life is full of subtle hints, fleeting opportunities, and silent lessons. If we are not paying attention, these secrets pass us by. But when we choose to slow down, to observe, and to listen, the world opens up to reveal its hidden wisdom, offering us the keys to deeper understanding and a richer existence.

It’s easy to overlook what is most important. But perhaps, the greatest secret is that the answers we seek have always been within us, in the quiet corners of our hearts, waiting to be acknowledged.
"),

    //18
    ("cosmic_dream.html", r#"
    <body style="font-family:Arial, sans-serif; line-height:1.6; max-width:800px; margin:auto; padding:20px; background-color:#0d0d0d; color:white;">

        <h1 style="margin-top: 0; text-align:center; color:#a1c4fd;">
            Cosmic Dream
        </h1>

        <p style="line-height: 2rem; text-align: justify;">
            Floating through the stars, time loses all meaning. There is no past, no future, only the endless expanse of the cosmos, stretching out in every direction. The stars twinkle softly like distant beacons, calling to us from the farthest reaches of the universe.
        </p>

        <div style="text-align:center; margin:20px 0;">
            #Media[deep_space_hum]
        </div>

        <p style="line-height: 2rem; text-align: justify;">
            The silence of space is profound, but there is a hum—a low, constant vibration that seems to resonate from the very fabric of the universe itself. It’s a sound you can feel more than hear, a deep, pulsating rhythm that echoes through your soul, connecting you to the cosmos in ways words cannot describe.
        </p>

        <p style="line-height: 2rem; text-align: justify;">
            As you drift further into the void, the planets and moons below you seem like distant memories. Their surface features are lost in the darkness, while nebulae swirl in brilliant clouds of color—vivid reds, purples, and blues. The stars, once so far away, now feel like part of you, and the universe seems infinite, without boundaries or limits.
        </p>

        <p style="line-height: 2rem; text-align: justify;">
            This is no longer just a dream; it’s an awakening. You realize that the stars are not separate from you, but part of your own being. The cosmos is not a cold, distant place—it’s a vast, living entity that you are a part of. In this moment, you are connected to everything, and the enormity of the universe becomes a source of comfort rather than fear.
        </p>

    </body>
    "#),

    //19
    ("unexpected_encounter.txt", "I turned the corner, and there it stood—watching.
At first, I thought my eyes were deceiving me, but no, it was real. A figure in the shadows, unmoving, with eyes that seemed to pierce right through me. The air felt heavier, charged with an unexplainable energy.

My heart raced, and my breath caught in my throat as I stood frozen, unsure whether to run or confront it. The figure didn’t speak, didn’t make a sound. It simply observed, its gaze locked onto mine. Every instinct told me to turn and flee, yet something about its presence kept me rooted to the spot.

Time seemed to stretch as I stood there, caught in a silent standoff. What was this? Was it a person, a figment of my imagination, or something else entirely? The uncertainty gnawed at me, making every second feel like an eternity. The figure remained still, and I began to wonder: What was it waiting for? Was it waiting for me to make the first move, or was it silently studying me, trying to understand who I was?

The encounter left me shaken, my mind racing with questions and possibilities. What I had just experienced, I couldn't explain—only that it was something beyond ordinary.
"),

    //20
    ("old_map.html", r#"
<body style="font-family:Georgia, serif; line-height:1.8; max-width:800px; margin:auto; padding:20px; background-color:#f4f4f4; color:#333;">

    <h1 style="margin-top: 0; text-align:center; color:#4f6d7a;">
        The Old Map
    </h1>

    <p style="line-height: 2rem; text-align: justify;">
        A parchment with faded ink, hinting at forgotten treasures. The map, weathered by time, seems almost alive—its corners curled, the ink smudged in places, but the paths and symbols still faintly visible. It’s as if it has been waiting, hidden in the shadows, for someone to uncover its secrets.
    </p>

    <div style="text-align:center; margin:20px 0;">
        #Media[hidden_cave]
    </div>

    <p style="line-height: 2rem; text-align: justify;">
        The map seems to depict a landscape that no longer exists—at least, not in any way familiar to the modern world. Ancient landmarks, now lost to time, are marked with symbols and words that speak of hidden caverns, buried riches, and dangers that once haunted the land.
    </p>

    <p style="line-height: 2rem; text-align: justify;">
        The more you study it, the more details emerge. A trail leading through dense forests, across rivers that no longer flow, and into a forgotten valley where a hidden cave lies waiting to be discovered. But something about the map feels strange. There’s an aura around it, a sense that it was meant for someone else—someone from the past, who left this map behind, hoping it would one day be found by the right person.
    </p>

    <p style="line-height: 2rem; text-align: justify;">
        What lies in that cave? Is it treasure, or something more dangerous? The map gives no answers, only more questions. Yet, there’s an undeniable pull—a calling that beckons you to follow its path, to find the treasure that history has left behind.
    </p>

</body>
"#),

    //21
    ("music_of_the_wind.txt", "If you listen closely, the wind tells a story.
It whispers through the trees, carrying with it tales from distant places—stories of forgotten lands, of adventures long past. The wind is not just air in motion; it is a storyteller, speaking in soft murmurs that only those who truly listen can understand.

At times, it carries with it the sound of morning birds, their songs harmonizing with the breeze. Each note seems to float in the air, like the wind itself is playing a melody composed in the farthest reaches of nature.

If you stop and close your eyes, you might hear it—the rustling leaves, the soft sighs as the wind weaves through the branches, and the delicate chirping of birds greeting the dawn. In that moment, the world fades away, and you are transported into the rhythm of the wind, where stories unfold in the quiet of nature.

The wind’s story is ever-changing, constantly moving, shifting with the seasons. One moment it carries the scent of spring, the next it brings the chill of winter’s approach. But through it all, it remains a constant storyteller, one whose tale is never fully told, yet always waiting to be heard.
"),

    //22
    ("forgotten_poem.html", r#"
<body style="font-family:Georgia, serif; line-height:1.8; max-width:800px; margin:auto; padding:20px; background-color:#fdf6e3; color:#333;">

    <h1 style="margin-top: 0; text-align:center; color:#6a4e5a;">
        The Forgotten Poem
    </h1>

    <p style="line-height: 2rem; text-align: justify;">
        A verse found in an old book, speaking of love lost. The pages were yellowed with age, the ink faded but still legible enough to capture the heart of the reader. It was as if the poem had been waiting, hidden for centuries, to speak its truth to someone who would understand.
    </p>

    <div style="text-align:center; margin:20px 0;">
        #Media[library_wonder]
    </div>

    <p style="line-height: 2rem; text-align: justify;">
        The poem speaks of a love that once bloomed brightly, a love that filled the air with warmth and light. But time, as it does with all things, took its toll. The words speak of memories that linger like shadows, fading with each passing day, yet never fully disappearing. A love that was once strong, now reduced to the echoes of the past.
    </p>

    <p style="line-height: 2rem; text-align: justify;">
        "What is love," the poem asks, "but a fleeting flame, one that burns bright only for a moment, before it is extinguished by time?" The words are haunting, a reminder that love, in its purest form, is both beautiful and tragic. It is something that can never be fully possessed, only remembered.
    </p>

    <p style="line-height: 2rem; text-align: justify;">
        As you read the lines, you can almost hear the voice of the poet, speaking softly in the silence of a forgotten library, surrounded by the dust of ages. The poem's message is timeless: that love, no matter how fleeting, leaves an indelible mark on the heart.
    </p>

</body>
"#),

    //23
    ("desert_whispers.html", r#"
<body style="font-family:Georgia, serif; line-height:1.8; max-width:800px; margin:auto; padding:20px; background-color:#f4e1c1; color:#4a2c2a;">

    <h1 style="margin-top: 0; text-align:center; color:#c68e17;">
        Desert Whispers
    </h1>

    <p style="line-height: 2rem; text-align: justify;">
        They say the dunes remember those who dare to cross. The desert, vast and unyielding, holds the footprints of those who venture across its endless sands. With each gust of wind, the dunes shift and change, as if the desert itself is keeping track of every soul who has passed through.
    </p>

    <div style="text-align:center; margin:20px 0;">
        #Media[desert_oasis]
    </div>

    <p style="line-height: 2rem; text-align: justify;">
        The heat is oppressive, but there is something else—something almost mystical about the desert. The silence, broken only by the occasional whisper of wind, seems to echo with memories. The desert holds secrets, secrets that only those brave enough to journey into its heart can begin to understand.
    </p>

    <p style="line-height: 2rem; text-align: justify;">
        The winds carry not just sand but stories—stories of travelers, merchants, and wanderers who sought solace in its endless expanse, only to disappear without a trace. Some say the dunes take what they want, hiding the past beneath layers of sand.
    </p>

    <div style="text-align:center; margin:20px 0;">
        #Media[city_ambience]
    </div>

    <p style="line-height: 2rem; text-align: justify;">
        As the sun sets, the desert transforms. The heat gives way to a cool breeze, and the once oppressive landscape is now a place of quiet reflection. The city, far off in the distance, hums with life, but here, in the vastness of the desert, there is only the wind and the whispers it carries.
    </p>
</body>
"#),

    //24
    ("cyber_rebellion.html", r#"<html>
<body style="font-family: 'Courier New', monospace; line-height: 1.8; max-width: 900px; margin: auto; padding: 20px; background-color: #121212; color: #e0e0e0;">

    <h1 style="margin-top: 0; text-align: center; color: #ff4081;">Cyber Rebellion</h1>

    <p style="line-height: 2rem; text-align: justify;">
        In the neon glow of the city, beneath the hum of digital billboards and the flicker of flickering lights, they planned their uprising. The city was a sprawling mass of concrete and steel, its skyline dominated by towering skyscrapers and holographic advertisements. But beneath the surface, the rebellion simmered—a movement of those who were tired of being controlled by the powerful corporations that ran everything.
    </p>

    <div style="text-align:center; margin:20px 0;">
        #Media[futuristic_city]
    </div>

    <p style="line-height: 2rem; text-align: justify;">
        The streets were alive with the pulse of technology, but there was a deeper current, one that thrived in the shadows. They met in secret, exchanging encrypted messages, their faces hidden behind augmented reality masks. The world above was bright, but the underground was dark, where those who had nothing left to lose could finally be heard.
    </p>

    <p style="line-height: 2rem; text-align: justify;">
        "The system is broken," they whispered, "but we can rebuild it, or tear it down, depending on how we choose to act." Their voices were determined, their hearts filled with a fire that could not be extinguished. The future was uncertain, but they were ready to fight for a new world, one free from the control of the corporations.
    </p>

    <div style="text-align:center; margin:20px 0;">
        #Media[train_journey]
    </div>

    <p style="line-height: 2rem; text-align: justify;">
        As the city’s pulse echoed around them, they boarded the trains that would take them to their destinations—places where they would lay the groundwork for their rebellion. The train carriages were silent, save for the hum of the electric engines and the occasional chatter of those who shared the same dream. The journey was long, but every mile brought them closer to a future they could only imagine.
    </p>

</body>
"#),

    //25
    ("ghostly_echoes.txt", "Ghostly Echoes

In the silence, I heard a voice—one that should not be there. It came from somewhere deep within the shadows, an unsettling whisper that seemed to echo through the very walls. I stood frozen, unsure whether I was hearing things or if something—or someone—was truly there.

I tried to dismiss the sound, but it grew louder, more distinct. It was a voice, faint yet unmistakable, calling out my name in a low, rasping tone that sent a chill down my spine. There was no one around, no one in sight, yet the voice persisted, a spectral presence that seemed to come from beyond.

#Media[haunted_whispers]

I moved cautiously through the room, my steps careful and deliberate, but the air felt thick, heavy with an eerie energy. The walls seemed to pulse with the sound, and I couldn't shake the feeling that I was not alone. Every corner I turned, every shadow I passed, I felt eyes upon me, unseen but ever-watchful.

The voice grew more insistent, like a distant memory clawing its way into my mind, and I knew then that whatever it was, it was not of this world. The haunted whispers lingered long after the night had passed, and I would never forget the chill they left in my soul.
"),
];


pub const IMAGE_PATHS: [(&str, &str, &str); 15] = [
    ("the_banana.html", "banana", "/content_objects/content_images/banana.png"),
    ("forbidden_text.html", "do_not_search_this", "/content_objects/content_images/do_not_search_this.jpg"),
    ("mountain_panoramas.html", "sparkling_snow", "/content_objects/content_images/sparkling_snow.jpg"),
    ("bigfoot_sighting.html", "big_foot", "/content_objects/content_images/big_foot.png"),
    ("famous_quote.html", "shakespeare", "/content_objects/content_images/shakespeare.png"),
    ("travel_dream.html", "tropical_paradise", "/content_objects/content_images/tropical_paradise.jpg"),
    ("astronomy_facts.html", "sunlight", "/content_objects/content_images/sunlight.jpg"),
    ("forest_mysteries.html", "forest_story", "/content_objects/content_images/forest_story.jpg"),
    ("city_lights.html", "city_night", "/content_objects/content_images/city_night.jpg"),
    ("lost_artifact.html", "ancient_artifact", "/content_objects/content_images/ancient_artifact.jpg"),
    ("forgotten_poem.html", "library_wonder", "/content_objects/content_images/library_wonder.jpg"),
    ("old_map.html", "hidden_cave", "/content_objects/content_images/hidden_cave.jpg"),
    ("desert_whispers.html", "desert_oasis", "/content_objects/content_images/desert_oasis.jpg"),
    ("cyber_rebellion.html", "futuristic_city", "/content_objects/content_images/futuristic_city.jpg"),
    ("abandoned_castle.jpeg", "abandoned_castle","/content_objects/content_images/abandoned_castle.jpg"),
];

/*pub const AUDIO_PATHS: [(&str, &str); 4] = [
    ("city_ambience", "path/to/city_ambience.mp3"),
    ("train_journey", "path/to/train_journey.mp3"),
    ("haunted_whispers", "path/to/haunted_whispers.mp3"),
    ("deep_space_hum", "path/to/deep_space_hum.mp3"),
];*/