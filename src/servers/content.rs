use std::collections::HashMap;
use rand::random;

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
    ("the_banana.html", r#"<html>
<head>
    <title>The Banana</title>
</head>
<body style="font-family:Arial, sans-serif; line-height:1.6; max-width:800px; margin:auto; padding:20px;">

    <h1 style="text-align:center;">The Banana</h1>

    <p>
        The banana is one of the most widely consumed fruits in the world. It is known for its sweet taste, soft texture, and nutritional benefits.
    </p>

    #Media[banana]

    <p>
        Originating from Southeast Asia, bananas are now grown in many tropical and subtropical regions. They are a rich source of potassium, vitamins, and fiber, making them an essential part of a healthy diet.
    </p>

    <p>
        Bananas are often eaten fresh, but they can also be used in smoothies, baked goods, and even savory dishes. Their natural sweetness makes them a great alternative to refined sugar in recipes.
    </p>

    <p>
        Beyond their culinary uses, bananas have cultural significance in many societies. In some places, the banana leaf is used as a natural plate, while in others, the fruit plays a role in religious ceremonies and traditions.
    </p>

    <p>
        Whether eaten alone, blended into a smoothie, or baked into a cake, the banana remains a favorite fruit worldwide.
    </p>

</body>
</html>
"#),

    //3
    ("forbidden_text.html", r#"<html>
<head>
    <title>Forbidden Text</title>
</head>
<body style="font-family:Arial, sans-serif; line-height:1.6; max-width:800px; margin:auto; padding:20px;">

    <h1 style="text-align:center; color:red;">⚠ Forbidden Text ⚠</h1>

    <p>
        There exist certain texts that have been hidden from the public eye for generations. Some say they contain knowledge too dangerous to be shared. Others believe they are mere superstition.
    </p>

    #Media[do_not_search_this]

    <p>
        The origins of this mysterious writing remain unknown. Fragments have surfaced in ancient libraries, secret societies, and encrypted manuscripts, yet no one has ever deciphered the full meaning.
    </p>

    <p>
        Many who have attempted to read the forbidden text speak of strange occurrences—whispers in the dark, flickering lights, and an overwhelming sense of unease. Could it all be coincidence, or is there something truly hidden within these words?
    </p>

    <p>
        Some scholars warn that certain knowledge is best left undiscovered. But for those who are curious, the question remains: will you dare to seek the truth, or will you turn back before it's too late?
    </p>

</body>
</html>
"#),

    //4
    ("phrases_by_lillo.txt", r#"Phrases by Lillo:
- a lack of belief in free will is the antidote to hate and judgement
- il disordine è tale finche non viene ordinato
- if you have to ask if you’re a member of a group, you’re probably not."#),

    //5
    ("mountain_panoramas.html", r#"<html>
<head>
    <title>Mountain Panoramas</title>
</head>
<body style="font-family:Arial, sans-serif; line-height:1.6; max-width:800px; margin:auto; padding:20px;">

    <h1 style="text-align:center;">Breathtaking Mountain Panoramas</h1>

    <p>
        Some of the most stunning views on Earth can be found in the mountains. Whether covered in lush greenery or blanketed in fresh snow, these landscapes offer a sense of peace and wonder.
    </p>

    <p>
        Walking along a mountain trail, the crisp air fills your lungs. The silence is only broken by the rustling of leaves and the distant call of a bird. Each step brings you closer to a perfect view, one that few get to witness.
    </p>

    #Media[sparkling_snow]

    <p>
        Sitting in the middle of a forest clearing, you gaze at the untouched snow glistening under the sun. The world feels still, frozen in time, as nature displays its beauty in the simplest of ways.
    </p>

    <p>
        Whether you seek adventure or solitude, the mountains have something for everyone. They remind us that the best panoramas are often right beside us, waiting to be discovered.
    </p>

</body>
</html>
"#),

    //6
    ("bigfoot_sighting.html", r#"<html>
<head>
    <title>Bigfoot Sighting Report</title>
</head>
<body style="font-family:Arial, sans-serif; line-height:1.6; max-width:800px; margin:auto; padding:20px;">

    <h1 style="text-align:center; color:brown;">Bigfoot Sighting Report</h1>

    <p><strong>Location:</strong> Dense forest near Willow Creek, California</p>
    <p><strong>Date and Time:</strong> December 12, 2024, 4:45 PM</p>

    #Media[big_foot]

    <h2>Witness Report</h2>

    <p>
        While hiking along an isolated trail, approximately 5 miles from the nearest road, I encountered an unusual figure standing roughly 50 yards away in a clearing.
    </p>

    <p>
        The figure was enormous, standing between 7 and 8 feet tall, with broad shoulders and a heavily muscled frame. Its body appeared to be covered in dark, shaggy hair, likely black or very dark brown, and it moved with a distinct upright, bipedal gait.
    </p>

    <p>
        For a few moments, we stood frozen—me, trying to process what I was seeing, and the creature, seemingly studying me in return. Then, with a slow yet deliberate movement, it turned and disappeared into the dense undergrowth, leaving behind only a series of deep footprints in the soft earth.
    </p>

    <p>
        Could it have been a hoax? Perhaps. But the eerie silence that followed, the unsettling weight of the moment—it felt real. Too real.
    </p>

</body>
</html>
"#),

    //7
    ("a_cat_life.txt", "A day in the life of a cat: sleep, eat, stare at nothing, and repeat."),

    //8
    ("famous_quote.html", r#"<html>
<head>
    <title>Famous Quote</title>
</head>
<body style="font-family:Arial, sans-serif; line-height:1.6; max-width:800px; margin:auto; padding:20px;">

    <h1 style="text-align:center;">To Be or Not to Be</h1>

    <p style="font-size:1.2em;">
        "To be, or not to be, that is the question."
        This famous line from Shakespeare’s *Hamlet* has resonated through the ages, posing one of life’s most profound questions. It speaks to the internal struggle, the contemplation of existence itself, and the choice between enduring suffering or ending it.
    </p>

    #Media[shakespeare]

    <p>
        Shakespeare’s *Hamlet* is one of the most well-known works of literature, and this quote encapsulates the torment of the play's protagonist. As Hamlet debates the merit of life and death, his words echo through time, exploring the universal theme of human existence and its meaning.
    </p>

    <p>
        Even today, over 400 years after it was written, the question remains relevant. We all face moments of doubt, moments when we question our purpose, our place in the world, and the choices we must make.
    </p>

    <p>
        Perhaps the real question is not whether to live or die, but how we live—how we embrace the moments of uncertainty and continue to move forward despite them.
    </p>

</body>
</html>
"#),

    //9
    ("recipe_for_happiness.txt", r#"Take one sunny day,
Add a sprinkle of laughter,
Mix in some good company,
And serve with warm smiles."#),

    //10
    ("travel_dream.html", r#"<html>
<head>
    <title>Travel Dream</title>
</head>
<body style="font-family:Arial, sans-serif; line-height:1.6; max-width:800px; margin:auto; padding:20px; background-color:#f0f8ff;">

    <h1 style="text-align:center; color:#2a8c8d;">A Dream of Travel</h1>

    <p>
        Imagine waking up to the sound of waves gently crashing against the shore, each wave a soft lullaby from the ocean. As the morning sun rises, a warm breeze carries the scent of saltwater and tropical flowers, filling your lungs with fresh air.
    </p>

    #Media[tropical_paradise]

    <p>
        The sky is painted in shades of pink and orange as the sun rises over turquoise waters. The horizon stretches out endlessly, offering a view so serene that it almost feels like a dream itself. You can hear the rustling of palm trees swaying in the breeze, their leaves whispering secrets of the tropical paradise they call home.
    </p>

    <p>
        For a moment, everything feels still. You sit back, close your eyes, and just breathe in the beauty of the world around you. This is a place where time slows down, where worries disappear, and all that remains is peace and wonder.
    </p>

    <p>
        This dream is not far from reality; it exists in the hearts of those who long to explore, to escape, and to find solace in the arms of nature. And perhaps, one day, you’ll find yourself there—waking up to the sound of waves and the promise of a new adventure.
    </p>

</body>
</html>
"#),

    //11
    ("astronomy_facts.html", r#"<html>
<head>
    <title>Astronomy Facts</title>
</head>
<body style="font-family:Arial, sans-serif; line-height:1.6; max-width:800px; margin:auto; padding:20px; background-color:#e8f0f8;">

    <h1 style="text-align:center; color:#1f5d71;">Did You Know? Astronomy Facts</h1>

    <p style="font-size:1.2em;">
        Did you know that the light from the Sun takes about 8 minutes to reach Earth? This incredible fact reveals just how vast and time-consuming space can be.
    </p>

    #Media[sunlight]

    <p>
        The distance between the Earth and the Sun is approximately 93 million miles (150 million kilometers). Despite the vast expanse, light travels incredibly fast—at 186,000 miles per second (299,792 kilometers per second).
    </p>

    <p>
        This means that when you look at the Sun, you're actually seeing it as it was 8 minutes ago. Imagine how far light travels in just a short span of time! And the fact that light from other stars can take millions or even billions of years to reach us is a humbling thought.
    </p>

    <p>
        Astronomy is full of mind-boggling facts like this, reminding us of the immense scale of the universe. So, the next time you look up at the sky, remember that the light you see today might have traveled a long way to get to you.
    </p>

</body>
</html>
"#),

    //12
    ("forest_mysteries.html", r#"<html>
<head>
    <title>Forest Mysteries</title>
</head>
<body style="font-family:Arial, sans-serif; line-height:1.6; max-width:800px; margin:auto; padding:20px; background-color:#f4f7f2;">

    <h1 style="text-align:center; color:#3e6e5f;">Forest Mysteries</h1>

    <p>
        The forest is alive with secrets, each tree and rustling leaf hiding a story of ancient times. Walk beneath the canopy of green, and you may feel the weight of history in the air, as if the trees themselves are whispering forgotten tales.
    </p>

    #Media[forest_story]

    <p>
        Listen closely to the rustling leaves, for in the sounds of the forest, there may be more than meets the eye. Every step you take could lead you closer to uncovering a hidden mystery—a forgotten path, a long-lost relic, or the echo of a voice carried on the wind.
    </p>

    <p>
        The forest holds its secrets well, but to those who seek, it offers glimpses of its hidden world. From the flutter of wings in the distance to the sudden stillness of the air, the forest keeps its story hidden, just waiting for someone to listen.
    </p>

    <p>
        Venture deeper, and perhaps you’ll find the key to unlocking its mysteries. But beware: not all secrets are meant to be uncovered. Some stories are better left untold, buried in the heart of the forest forever.
    </p>

</body>
</html>
"#),

    //13
    ("tech_innovations.txt", r#"The rise of AI is transforming industries:
From healthcare to space exploration,
The future is already here."#),

    //14
    ("city_lights.html", r#"<html>
<head>
    <title>City Lights</title>
</head>
<body style="font-family:Arial, sans-serif; line-height:1.6; max-width:800px; margin:auto; padding:20px; background-color:#e9e9e9;">

    <h1 style="text-align:center; color:#4c4f5c;">City Lights</h1>

    <p>
        Standing at the rooftop, I watch as the city glows with a million lights, each one representing a story, a life, a moment in time. The streets are alive with movement, but from up here, it all seems so peaceful, almost like a glowing constellation beneath me.
    </p>

    #Media[city_night]

    <p>
        The skyline stretches endlessly, a jagged line of towering buildings punctuated by the flicker of neon signs and streetlights. It’s as though the city never sleeps, always awake and vibrant, pulsating with energy even in the dead of night.
    </p>

    <p>
        As I gaze down at the sea of lights, I can’t help but wonder who else is out there, watching the same view, lost in their own thoughts. The city connects us all, even when we’re alone on this rooftop, with the bright lights below casting shadows on our dreams.
    </p>

    <p>
        The distant hum of traffic is like a soft melody, soothing in its predictability, while the occasional car honk or laughter from a nearby street cafe reminds me that life continues below. The city feels like an eternal dance, its rhythm set by the heartbeat of its inhabitants.
    </p>

</body>
</html>
"#),

    //15
    ("winter_tale.txt", r#"The first snow of the year falls gently,
Covering the world in a blanket of white.
A serene and magical moment."#),

    //16
    ("lost_artifact.html", r#"<html>
<head>
    <title>Lost Artifact</title>
</head>
<body style="font-family:Arial, sans-serif; line-height:1.6; max-width:800px; margin:auto; padding:20px; background-color:#fafafa;">

    <h1 style="text-align:center; color:#4d4b39;">The Lost Artifact</h1>

    <p>
        A mysterious artifact was discovered in the vast expanse of the desert, buried beneath the shifting sands. Its origins are unknown, and the symbols etched onto its surface remain undeciphered to this day. Some believe it holds the key to an ancient civilization, long forgotten by time.
    </p>

    #Media[ancient_artifact]

    <p>
        The artifact, made of an unknown metal that gleams even under the harsh desert sun, has baffled archaeologists and historians for years. Some say it’s a relic from a long-lost kingdom, others argue it could be an alien artifact, left behind by visitors from beyond our world.
    </p>

    <p>
        As experts continue to study it, no one can explain the strange symbols that cover its surface. They don’t match any known script, leaving researchers with more questions than answers. Is this an ancient relic from a forgotten people, or something far more mysterious? Only time will tell.
    </p>

    <p>
        Until then, the artifact remains a symbol of humanity's unending curiosity and the allure of the unknown. It stands as a reminder that there are still mysteries in this world—some that may never be solved.
    </p>

</body>
</html>
"#),

    //17
    ("hidden_wisdom.txt", "The greatest secrets are not kept, but overlooked.
In the hustle of daily life, we often miss the things that truly matter—hidden in plain sight, waiting to be discovered. The wisdom we seek is often not hidden in distant lands or ancient texts, but right before our eyes, in the small moments, the quiet whispers of nature, or the simple truths that we forget to acknowledge.

Life is full of subtle hints, fleeting opportunities, and silent lessons. If we are not paying attention, these secrets pass us by. But when we choose to slow down, to observe, and to listen, the world opens up to reveal its hidden wisdom, offering us the keys to deeper understanding and a richer existence.

It’s easy to overlook what is most important. But perhaps, the greatest secret is that the answers we seek have always been within us, in the quiet corners of our hearts, waiting to be acknowledged.
"),

    //18
    ("cosmic_dream.html", r#"<html>
<head>
    <title>Cosmic Dream</title>
</head>
<body style="font-family:Arial, sans-serif; line-height:1.6; max-width:800px; margin:auto; padding:20px; background-color:#0d0d0d; color:white;">

    <h1 style="text-align:center; color:#a1c4fd;">Cosmic Dream</h1>

    <p>
        Floating through the stars, time loses all meaning. There is no past, no future, only the endless expanse of the cosmos, stretching out in every direction. The stars twinkle softly like distant beacons, calling to us from the farthest reaches of the universe.
    </p>

    #Media[deep_space_hum]

    <p>
        The silence of space is profound, but there is a hum—a low, constant vibration that seems to resonate from the very fabric of the universe itself. It’s a sound you can feel more than hear, a deep, pulsating rhythm that echoes through your soul, connecting you to the cosmos in ways words cannot describe.
    </p>

    <p>
        As you drift further into the void, the planets and moons below you seem like distant memories. Their surface features are lost in the darkness, while nebulae swirl in brilliant clouds of color—vivid reds, purples, and blues. The stars, once so far away, now feel like part of you, and the universe seems infinite, without boundaries or limits.
    </p>

    <p>
        This is no longer just a dream; it’s an awakening. You realize that the stars are not separate from you, but part of your own being. The cosmos is not a cold, distant place—it’s a vast, living entity that you are a part of. In this moment, you are connected to everything, and the enormity of the universe becomes a source of comfort rather than fear.
    </p>

</body>
</html>
"#),

    //19
    ("unexpected_encounter.txt", "I turned the corner, and there it stood—watching.
At first, I thought my eyes were deceiving me, but no, it was real. A figure in the shadows, unmoving, with eyes that seemed to pierce right through me. The air felt heavier, charged with an unexplainable energy.

My heart raced, and my breath caught in my throat as I stood frozen, unsure whether to run or confront it. The figure didn’t speak, didn’t make a sound. It simply observed, its gaze locked onto mine. Every instinct told me to turn and flee, yet something about its presence kept me rooted to the spot.

Time seemed to stretch as I stood there, caught in a silent standoff. What was this? Was it a person, a figment of my imagination, or something else entirely? The uncertainty gnawed at me, making every second feel like an eternity. The figure remained still, and I began to wonder: What was it waiting for? Was it waiting for me to make the first move, or was it silently studying me, trying to understand who I was?

The encounter left me shaken, my mind racing with questions and possibilities. What I had just experienced, I couldn't explain—only that it was something beyond ordinary.
"),

    //20
    ("old_map.html", r#"<html>
<head>
    <title>Old Map</title>
</head>
<body style="font-family:Georgia, serif; line-height:1.8; max-width:800px; margin:auto; padding:20px; background-color:#f4f4f4; color:#333;">

    <h1 style="text-align:center; color:#4f6d7a;">The Old Map</h1>

    <p>
        A parchment with faded ink, hinting at forgotten treasures. The map, weathered by time, seems almost alive—its corners curled, the ink smudged in places, but the paths and symbols still faintly visible. It’s as if it has been waiting, hidden in the shadows, for someone to uncover its secrets.
    </p>

    #Media[hidden_cave]

    <p>
        The map seems to depict a landscape that no longer exists—at least, not in any way familiar to the modern world. Ancient landmarks, now lost to time, are marked with symbols and words that speak of hidden caverns, buried riches, and dangers that once haunted the land.
    </p>

    <p>
        The more you study it, the more details emerge. A trail leading through dense forests, across rivers that no longer flow, and into a forgotten valley where a hidden cave lies waiting to be discovered. But something about the map feels strange. There’s an aura around it, a sense that it was meant for someone else—someone from the past, who left this map behind, hoping it would one day be found by the right person.
    </p>

    <p>
        What lies in that cave? Is it treasure, or something more dangerous? The map gives no answers, only more questions. Yet, there’s an undeniable pull—a calling that beckons you to follow its path, to find the treasure that history has left behind.
    </p>

</body>
</html>
"#),

    //21
    ("music_of_the_wind.txt", "If you listen closely, the wind tells a story.
It whispers through the trees, carrying with it tales from distant places—stories of forgotten lands, of adventures long past. The wind is not just air in motion; it is a storyteller, speaking in soft murmurs that only those who truly listen can understand.

At times, it carries with it the sound of morning birds, their songs harmonizing with the breeze. Each note seems to float in the air, like the wind itself is playing a melody composed in the farthest reaches of nature.

If you stop and close your eyes, you might hear it—the rustling leaves, the soft sighs as the wind weaves through the branches, and the delicate chirping of birds greeting the dawn. In that moment, the world fades away, and you are transported into the rhythm of the wind, where stories unfold in the quiet of nature.

The wind’s story is ever-changing, constantly moving, shifting with the seasons. One moment it carries the scent of spring, the next it brings the chill of winter’s approach. But through it all, it remains a constant storyteller, one whose tale is never fully told, yet always waiting to be heard.
"),

    //22
    ("forgotten_poem.html", r#"<html>
<head>
    <title>Forgotten Poem</title>
</head>
<body style="font-family:Georgia, serif; line-height:1.8; max-width:800px; margin:auto; padding:20px; background-color:#fdf6e3; color:#333;">

    <h1 style="text-align:center; color:#6a4e5a;">The Forgotten Poem</h1>

    <p>
        A verse found in an old book, speaking of love lost. The pages were yellowed with age, the ink faded but still legible enough to capture the heart of the reader. It was as if the poem had been waiting, hidden for centuries, to speak its truth to someone who would understand.
    </p>

    #Media[library_wonder]

    <p>
        The poem speaks of a love that once bloomed brightly, a love that filled the air with warmth and light. But time, as it does with all things, took its toll. The words speak of memories that linger like shadows, fading with each passing day, yet never fully disappearing. A love that was once strong, now reduced to the echoes of the past.
    </p>

    <p>
        "What is love," the poem asks, "but a fleeting flame, one that burns bright only for a moment, before it is extinguished by time?" The words are haunting, a reminder that love, in its purest form, is both beautiful and tragic. It is something that can never be fully possessed, only remembered.
    </p>

    <p>
        As you read the lines, you can almost hear the voice of the poet, speaking softly in the silence of a forgotten library, surrounded by the dust of ages. The poem's message is timeless: that love, no matter how fleeting, leaves an indelible mark on the heart.
    </p>

</body>
</html>
"#),

    //23
    ("desert_whispers.html", r#"<html>
<head>
    <title>Desert Whispers</title>
</head>
<body style="font-family:Georgia, serif; line-height:1.8; max-width:800px; margin:auto; padding:20px; background-color:#f4e1c1; color:#4a2c2a;">

    <h1 style="text-align:center; color:#c68e17;">Desert Whispers</h1>

    <p>
        They say the dunes remember those who dare to cross. The desert, vast and unyielding, holds the footprints of those who venture across its endless sands. With each gust of wind, the dunes shift and change, as if the desert itself is keeping track of every soul who has passed through.
    </p>

    #Media[desert_oasis]

    <p>
        The heat is oppressive, but there is something else—something almost mystical about the desert. The silence, broken only by the occasional whisper of wind, seems to echo with memories. The desert holds secrets, secrets that only those brave enough to journey into its heart can begin to understand.
    </p>

    <p>
        The winds carry not just sand but stories—stories of travelers, merchants, and wanderers who sought solace in its endless expanse, only to disappear without a trace. Some say the dunes take what they want, hiding the past beneath layers of sand.
    </p>

    #Media[city_ambience]

    <p>
        As the sun sets, the desert transforms. The heat gives way to a cool breeze, and the once oppressive landscape is now a place of quiet reflection. The city, far off in the distance, hums with life, but here, in the vastness of the desert, there is only the wind and the whispers it carries.
    </p>

</body>
</html>
"#),

    //24
    ("cyber_rebellion.html", r#"<html>
<head>
    <title>Cyber Rebellion</title>
</head>
<body style="font-family: 'Courier New', monospace; line-height: 1.8; max-width: 900px; margin: auto; padding: 20px; background-color: #121212; color: #e0e0e0;">

    <h1 style="text-align: center; color: #ff4081;">Cyber Rebellion</h1>

    <p>
        In the neon glow of the city, beneath the hum of digital billboards and the flicker of flickering lights, they planned their uprising. The city was a sprawling mass of concrete and steel, its skyline dominated by towering skyscrapers and holographic advertisements. But beneath the surface, the rebellion simmered—a movement of those who were tired of being controlled by the powerful corporations that ran everything.
    </p>

    #Media[futuristic_city]

    <p>
        The streets were alive with the pulse of technology, but there was a deeper current, one that thrived in the shadows. They met in secret, exchanging encrypted messages, their faces hidden behind augmented reality masks. The world above was bright, but the underground was dark, where those who had nothing left to lose could finally be heard.
    </p>

    <p>
        "The system is broken," they whispered, "but we can rebuild it, or tear it down, depending on how we choose to act." Their voices were determined, their hearts filled with a fire that could not be extinguished. The future was uncertain, but they were ready to fight for a new world, one free from the control of the corporations.
    </p>

    #Media[train_journey]

    <p>
        As the city’s pulse echoed around them, they boarded the trains that would take them to their destinations—places where they would lay the groundwork for their rebellion. The train carriages were silent, save for the hum of the electric engines and the occasional chatter of those who shared the same dream. The journey was long, but every mile brought them closer to a future they could only imagine.
    </p>

</body>
</html>
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
    ("the_banana.html", "banana", "/static/content_objects/content_images/banana.png"),
    ("forbidden_text.html", "do_not_search_this", "/static/content_objects/content_images/do_not_search_this.jpg"),
    ("mountain_panoramas.html", "sparkling_snow", "/static/content_objects/content_images/do_not_search_this.jpg"),
    ("bigfoot_sighting.html", "big_foot", "/static/content_objects/content_images/big_foot.png"),
    ("famous_quote.html", "shakespeare", "/static/content_objects/content_images/shakespeare.png"),
    ("travel_dream.html", "tropical_paradise", "/static/content_objects/content_images/tropical_paradise.jpg"),
    ("astronomy_facts.html", "sunlight", "/static/content_objects/content_images/sunlight.jpg"),
    ("forest_mysteries.html", "forest_story", "/static/content_objects/content_images/forest_story.jpg"),
    ("city_lights.html", "city_night", "/static/content_objects/content_images/city_night.jpg"),
    ("lost_artifact.html", "ancient_artifact", "/static/content_objects/content_images/ancient_artifact.jpg"),
    ("forgotten_poem.html", "library_wonder", "/static/content_objects/content_images/library_wonder.jpg"),
    ("old_map.html", "hidden_cave", "/static/content_objects/content_images/hidden_cave.jpg"),
    ("desert_whispers.html", "desert_oasis", "/static/content_objects/content_images/desert_oasis.jpg"),
    ("cyber_rebellion.html", "futuristic_city", "/static/content_objects/content_images/futuristic_city.jpg"),
    ("abandoned_castle.jpeg", "abandoned_castle","/static/content_objects/content_images/abandoned_castle.jpg"),
];

/*pub const AUDIO_PATHS: [(&str, &str); 4] = [
    ("city_ambience", "path/to/city_ambience.mp3"),
    ("train_journey", "path/to/train_journey.mp3"),
    ("haunted_whispers", "path/to/haunted_whispers.mp3"),
    ("deep_space_hum", "path/to/deep_space_hum.mp3"),
];*/