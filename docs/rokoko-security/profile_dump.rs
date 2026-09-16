use rokoko::protocol::{config::{Config, Projection},params::{P_MICRO,P_TINY,P_SMALL,P_MEDIUM,P_LARGE}};
fn main() {
 eprintln!("sampler_mean_attempts={:.8}",rokoko::common::short_challenge::repetition_rate());
 for (name, root) in [("p-22", &*P_MICRO),("p-24", &*P_TINY),("p-26", &*P_SMALL),("p-28", &*P_MEDIUM),("p-30", &*P_LARGE)] {
  let mut current=Some(root); let mut round=0;
  while let Some(c)=current { round+=1;
   match c {
    Config::Sumcheck(s)=> { println!("{},{},{},{},{},{},{},{}",name,round,s.witness_height,s.witness_width,s.projection_ratio,s.projection_height,match &s.projection_recursion {Projection::Skip=>"skip",Projection::Fine(_)=>"fine",Projection::Coarse(_)=>"coarse"},s.basic_commitment_rank); current=s.next.as_deref(); },
    Config::Simple(s)=>{println!("{},{},{},{},{},{},simple,{}",name,round,s.witness_height,s.witness_width,s.projection_ratio,s.projection_height,s.basic_commitment_rank); current=None;},
    Config::Intermediate(_)=>panic!("unexpected intermediate config")
   }
  }
 }
}
