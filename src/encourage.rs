use rand::prelude::IndexedRandom;

pub fn random_encouragement() -> &'static str{
    let encouragements = [
        "焦らず、自分のペースで進めましょうね。",
        "無理せず、できることからで大丈夫ですよ。",
        "あなたならきっと大丈夫です！",
        "今日も一歩前進ですね。応援しています。",
        "疲れたら、少し休むのも大事ですよ。",
        "頑張りすぎないでくださいね。ことのははいつでも味方です。"
    ];

    encouragements.choose(&mut rand::rng()).unwrap()
}
pub fn random_topic() -> &'static str {
    let topics = [
        "ところで、最近ハマっていることはありますか？",
        "最近見た映画や本でおすすめはありますか？",
        "お休みの日はどんなふうに過ごされていますか？",
        "好きな食べ物を教えてください！",
        "最近チャレンジしたことがあれば、ぜひ聞かせてください！",
        "今日の天気、いい感じでしたか？"
    ];
    topics.choose(&mut rand::rng()).unwrap()
}
