import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';

const QueryResult = () => {
  const [querySuccess, setQuerySuccess] = useState(null);
  const { id } = useParams();

  useEffect(() => {
    const BASE_URL = 'http://localhost:3000/dataset';
    const url = `${BASE_URL}/${id}`;

    fetch(url)
      .then(response => response.json())
      .then(data => {
        if (data.web3 === null) {
          setQuerySuccess(false);
          return;
        } else {
            setQuerySuccess(data.web3.tx.hash);
        }
    })
      .catch(error => {
        console.error('Error fetching data:', error)
        setQuerySuccess(false);
      });
  }, [id]);

  if (querySuccess === null) {
    return <p>Loading...</p>;
  }

  return (
    <div>
      {querySuccess ? (
        <p style={{ color: 'green' }}>✅ The provided dataset is valid and stored on the blockchain system at transaction {querySuccess}</p>
      ) : (
        <p style={{ color: 'red' }}>❌ The provided dataset was not found</p>
      )}
    </div>
  );
};

export default QueryResult;
