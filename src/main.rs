use std::{env, ops::{AddAssign, DivAssign}, fs::File, io::prelude::*};
use orca::{App, LimitMethod};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use sentiment::{Analysis, Sentiment};

#[derive(Serialize, Deserialize)]
struct Credentials {
    id:       String,
    secret:   String,
    username: String,
    password: String,
}

#[derive(Debug)]
struct Score {
    score: f32,
    comp:  f32,
}

impl AddAssign<&Score> for Score {
    fn add_assign(&mut self, other: &Score) {
        self.score += other.score;
        self.comp += other.comp;
    }
}

impl DivAssign<f32> for Score {
    fn div_assign(&mut self, divisor: f32) {
        self.score /= divisor;
        self.comp /= divisor;
    }
}

impl Score {
    fn convert_analysis(analysis: &Analysis) -> Score {
        Score {
            score: analysis.score,
            comp:  analysis.comparative
        }
    }

    fn convert_sentiment(sentiment: &Sentiment) -> Score {
        Score {
            score: sentiment.score,
            comp:  sentiment.comparative
        }
    }
}

#[derive(Debug)]
struct AggregateScore {
    total:    Score,
    positive: Score,
    negative: Score
}

impl AddAssign<&AggregateScore> for &mut AggregateScore {
    fn add_assign(&mut self, other: &AggregateScore) {
        self.total += &other.total;
        self.positive += &other.positive;
        self.negative += &other.negative;
    }
}

impl DivAssign<f32> for &mut AggregateScore {
    fn div_assign(&mut self, divisor: f32) {
        self.total /= divisor;
        self.positive /= divisor;
        self.negative /= divisor;
    }
}

fn get_credentials() -> Result<Credentials> {
    let mut file = File::open("creds.json").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    serde_json::from_str::<Credentials>(contents.as_str())
}

fn get_app() -> App {
    let creds = get_credentials().unwrap();
    let mut redd = App::new("SubredditSentiments", "v0.1.0", &format!("/u/{}", creds.username)).unwrap();

    redd.authorize_script(&creds.id, &creds.secret, &creds.username, &creds.password).unwrap();

    return redd;
}

fn accum_stats(mut agg: &mut AggregateScore, comment: &str) {
    let analysis = sentiment::analyze(String::from(comment));

    let score = AggregateScore {
        total: Score::convert_analysis(&analysis),
        positive: Score::convert_sentiment(&analysis.positive),
        negative: Score::convert_sentiment(&analysis.negative)
    };

    agg += &score;
}

fn avg_stats(mut agg: &mut AggregateScore, runs: f32) {
    agg /= runs;
}

fn get_sub_sentiment(redd: &App, sub: &str, runs : usize) {
    let mut total = AggregateScore {
        total: Score {
            score: 0.,
            comp:  0.,
        },
        positive: Score {
            score: 0.,
            comp:  0.,
        },
        negative: Score {
            score: 0.,
            comp:  0.,
        },
    }; 
    /*
    for comment in redd.create_comment_stream(sub) {
        println!("{}: {}\n", comment.author, comment.body);
    }
    */
    for (idx, comment) in redd.create_comment_stream(sub).enumerate() {
        println!("Run {}: {}\n", idx, comment.body);
        if idx > runs {
            break;
        }
        accum_stats(&mut total, &comment.body);
        println!("Run {} end", idx);
    }

    avg_stats(&mut total, runs as f32);

    println!("Stats:\n{:#?}", total);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    assert!(args.len() == 3);

    let redd = get_app();
    
    get_sub_sentiment(&redd, &args[1], args[2].parse::<usize>().unwrap());
}
