# tokio tide

使用 tokio 实现 tide。

tide 的架构思想是我很喜欢的。虽然 actix-web 也透露着洋葱模型，但比起 tide 似乎有些大。aysnc-std 的周边又不如 tokio。所以这就产生了一个问题：简单的架构却用不了丰富的周边。这是很困扰的。

所以用 tokio 抄一遍 tide。练练手。