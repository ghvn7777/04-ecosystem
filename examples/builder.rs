use anyhow::Result;
use chrono::{DateTime, Datelike, Utc};
use derive_builder::Builder;

#[allow(unused)]
#[derive(Debug, Builder)]
#[builder(build_fn(name = "_priv_build"))] // 我们自己实现 build 方法
struct User {
    // 只要实现了 Into<String> 就可以当参数
    #[builder(setter(into))]
    name: String,

    // strip_option 表示去掉 Option 包装，设置时候不用 Some
    // default 提供默认值，默认是 None
    #[builder(setter(into, strip_option), default)]
    email: Option<String>,

    // 自定义 setter 函数解析
    #[builder(setter(custom))]
    dob: DateTime<Utc>,

    // 隐藏字段, age 不用设置
    #[builder(setter(skip))]
    age: u32,

    // 重命名字段，设置时候每个元素都会调用 skill 函数
    #[builder(default = "vec![]", setter(each(name = "skill", into)))]
    skills: Vec<String>,
}

fn main() -> Result<()> {
    let user = User::build()
        .name("Alice")
        .skill("programming")
        .skill("debugging")
        .email("tyr@awesome.com")
        .dob("1990-01-01T00:00:00Z")
        .build()?;

    println!("{:?}", user);

    Ok(())
}

impl User {
    pub fn build() -> UserBuilder {
        UserBuilder::default()
    }
}

impl UserBuilder {
    pub fn build(&self) -> Result<User> {
        let mut user = self._priv_build()?;
        user.age = (Utc::now().year() - user.dob.year()) as _;
        Ok(user)
    }
    pub fn dob(&mut self, value: &str) -> &mut Self {
        self.dob = DateTime::parse_from_rfc3339(value)
            .map(|dt| dt.with_timezone(&Utc))
            .ok();
        self
    }
}
