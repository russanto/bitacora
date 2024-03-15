import React from 'react';
import './App.css';
import QueryResult from './QueryResult';
import logo from './logo.png';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';

function App() {
  return (
    <Router>
      <div className="App">
        <header className="App-header">
          <img src={logo} className="App-logo" alt="logo" />
          <Routes>
            <Route path="/:id" element={<QueryResult />} />
          </Routes>
        </header>
      </div>
    </Router>
  );
}

export default App;
