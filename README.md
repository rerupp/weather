# Historical Weather Data
A collection of applications to gather and display historical weather data.

## Background

The original weather data project was written entirely in `Python`. It started out while
spending the winter in AZ. During several happy hours, long time visitors (snow birds),
kept mentioning it was the coldest winter they could remember. Some of them had
been coming to AZ for upwards to 20 years.

Not that I didn't believe what they were saying, but I began to wonder if there was some online 
service that would provide historical weather data. At the time I came across a site, *Dark
Sky*, that had a `REST` API providing daily weather history. Originally the weather data
was a collection of scripts with Excel but it quickly became a mess to get and show data. This
is when the original `Python` implementation began.

In 2020 *Dark Sky* was purchased by Apple and the API went away. That mostly ended the `Python` 
project. Other services at that time were either ones you had to pay for or the free services 
just didn't provide the same amount of information.

Back in May 2022 I came across an article that discussed the Linux communities decision to allow
`Rust` to be used within the kernel. I had read several articles on the language and was somewhat
surprised due to how (relatively) new the language was.

I spent some time going through *The Rust Programming Language* and came away wanting to explore
it in more detail. If I could come up with some type of project I could take her out for a spin 
so to speak.

I wondered if there were any new weather data services available online and came across the *Visual
Crossing* site. Similar to *Dark Sky*, access to the site was free (depending on usage) and 
it provided data similar to what had already collected with the `Python` programs. Thats when
the `Rust` version of the `Python` implementation began.

## Today
The `Python` and `Rust` projects were previously maintained in separate repositories on `GitHub`.
The `Rust` backend turned out to be very quick however the choices for a front-end are somewhat
lacking right now. There is a `Rust` based CLI that has the most functionality, it supports adding, 
viewing, and administration of the weather data history. There is a `Rust` based TUI written 
using the `ratatui` framework that somewhat mimics the original `Python` GUI but it really is a
poor implementation and the interface is pretty dated.

As I was looking around for a `Rust` GUI framework I came across the `PyO3` framework. It made 
me wonder if it could be used to have a `Python` front-end and `Rust` as thew backend. Originally
I had concerns `PyO3` would be similar to Java/JNDI or .Net/COM but to my surprise it was straight
forward and performance was great.

## The Repository Layout
The repository is divided into the following folders.

* the `python` folder contains `Python` code
* the `rust` folder contains the `Rust` code.

The `PyO3` bindings exist in the `rust/py_lib` folder. For a period of time it was in 
the `python` folder but that didn't feel right. The annoyance is that once the `Python` folder 
is setup with a virtual environment you need to go into the `rust/py_lib` folder, use 
`maturin` to create and install the `Python` bindings.

## Getting Started

Right now the easiest way to begin is build the `rust` project. Once that has been built 
there is the `weather` CLI that can be used to bootstrap the environment. You will then need to go
to `visualcrossing.com` and get an API key. They have a "sign up for free" link that will let 
you retrieve up to 1000 days of historical data per calendar day.

There are more details about getting started in each of the source folders.
